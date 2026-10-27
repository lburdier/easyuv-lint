//! UV layout preview as SVG: island outlines, overlapping faces in red, flipped faces in orange.

use easyuv::mesh::Mesh;
use std::collections::HashMap;
use std::fmt::Write;

const SIZE: f64 = 1000.0;
/// Below this many faces the full wireframe is drawn; above, only island outlines.
const WIRE_MAX: usize = 6000;

/// Faces that share UV space with another face, on a `res` grid over the UV bounds.
/// Cell centres lying exactly on an edge belong to no triangle, so neighbours never count.
fn overlapping_faces(mesh: &Mesh, uv: &[[f32; 2]], lo: [f64; 2], side: f64, res: usize) -> Vec<bool> {
    let mut owner = vec![u32::MAX; res * res];
    let mut hit = vec![false; mesh.face_count()];
    let k = res as f64 / side;
    for f in 0..mesh.face_count() {
        let r = mesh.face_range(f);
        for t in mesh.fan(f) {
            let p: [[f64; 2]; 3] = std::array::from_fn(|j| {
                let c = uv[r.start + t[j]];
                [(c[0] as f64 - lo[0]) * k, (c[1] as f64 - lo[1]) * k]
            });
            let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
            if area.abs() < 1e-12 {
                continue;
            }
            let s = area.signum();
            let edge = |a: [f64; 2], b: [f64; 2], x: f64, y: f64| (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0]);
            let x0 = p.iter().map(|q| q[0]).fold(f64::MAX, f64::min).floor().max(0.0) as usize;
            let x1 = (p.iter().map(|q| q[0]).fold(f64::MIN, f64::max).ceil().max(0.0) as usize).min(res - 1);
            let y0 = p.iter().map(|q| q[1]).fold(f64::MAX, f64::min).floor().max(0.0) as usize;
            let y1 = (p.iter().map(|q| q[1]).fold(f64::MIN, f64::max).ceil().max(0.0) as usize).min(res - 1);
            for cy in y0..=y1 {
                for cx in x0..=x1 {
                    let (x, y) = (cx as f64 + 0.5, cy as f64 + 0.5);
                    if s * edge(p[0], p[1], x, y) > 0.0 && s * edge(p[1], p[2], x, y) > 0.0 && s * edge(p[2], p[0], x, y) > 0.0 {
                        let cell = &mut owner[cy * res + cx];
                        if *cell == u32::MAX {
                            *cell = f as u32;
                        } else if *cell != f as u32 {
                            hit[*cell as usize] = true;
                            hit[f] = true;
                        }
                    }
                }
            }
        }
    }
    hit
}

/// Faces mirrored relative to the majority orientation of their island.
fn flipped_faces(mesh: &Mesh, uv: &[[f32; 2]]) -> Vec<bool> {
    let (fi, n) = easyuv::layout::uv_islands(mesh, uv);
    let area: Vec<f64> = (0..mesh.face_count())
        .map(|f| {
            let r = mesh.face_range(f);
            let c: Vec<[f64; 2]> = r.map(|i| [uv[i][0] as f64, uv[i][1] as f64]).collect();
            (0..c.len()).map(|i| c[i][0] * c[(i + 1) % c.len()][1] - c[(i + 1) % c.len()][0] * c[i][1]).sum::<f64>() * 0.5
        })
        .collect();
    let mut sign = vec![0.0; n];
    for (f, a) in area.iter().enumerate() {
        sign[fi[f] as usize] += a;
    }
    area.iter().enumerate().map(|(f, a)| a * sign[fi[f] as usize] < 0.0).collect()
}

pub fn uv_layout(mesh: &Mesh, uv: &[[f32; 2]]) -> String {
    // view: UV bounds, always including the 0-1 tile
    let (mut lo, mut hi) = ([0.0f64, 0.0], [1.0f64, 1.0]);
    for c in uv {
        lo = [lo[0].min(c[0] as f64), lo[1].min(c[1] as f64)];
        hi = [hi[0].max(c[0] as f64), hi[1].max(c[1] as f64)];
    }
    let side = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-9);
    let px = |c: [f32; 2]| ((c[0] as f64 - lo[0]) / side * SIZE, SIZE - (c[1] as f64 - lo[1]) / side * SIZE);
    let overlap = overlapping_faces(mesh, uv, lo, side, 512);
    let flipped = flipped_faces(mesh, uv);

    let poly = |out: &mut String, f: usize| {
        for (i, c) in mesh.face_range(f).enumerate() {
            let (x, y) = px(uv[c]);
            let _ = write!(out, "{}{:.1} {:.1}", if i == 0 { "M" } else { "L" }, x, y);
        }
        out.push('Z');
    };
    let (mut wire, mut red, mut orange) = (String::new(), String::new(), String::new());
    let full = mesh.face_count() <= WIRE_MAX;
    for f in 0..mesh.face_count() {
        if full {
            poly(&mut wire, f);
        }
        if flipped[f] {
            poly(&mut orange, f);
        } else if overlap[f] {
            poly(&mut red, f);
        }
    }
    // island outlines: UV edges used by a single face
    let key = |c: [f32; 2]| ((c[0] * 65536.0).round() as i64, (c[1] * 65536.0).round() as i64);
    let mut edges: HashMap<((i64, i64), (i64, i64)), (u32, [f32; 2], [f32; 2])> = HashMap::new();
    for f in 0..mesh.face_count() {
        let r = mesh.face_range(f);
        let n = r.len();
        for i in 0..n {
            let (a, b) = (uv[r.start + i], uv[r.start + (i + 1) % n]);
            let (ka, kb) = (key(a), key(b));
            let k = if ka < kb { (ka, kb) } else { (kb, ka) };
            edges.entry(k).and_modify(|e| e.0 += 1).or_insert((1, a, b));
        }
    }
    let mut outline = String::new();
    for (count, a, b) in edges.values() {
        if *count == 1 {
            let ((x0, y0), (x1, y1)) = (px(*a), px(*b));
            let _ = write!(outline, "M{x0:.1} {y0:.1}L{x1:.1} {y1:.1}");
        }
    }
    let (t0, t1) = (px([0.0, 1.0]), px([1.0, 0.0]));
    format!(
        concat!(
            r#"<svg class="uvmap" viewBox="-8 -8 1016 1016" xmlns="http://www.w3.org/2000/svg" role="img">"#,
            r#"<rect x="{tx:.1}" y="{ty:.1}" width="{tw:.1}" height="{th:.1}" class="tile"/>"#,
            r#"<path d="{wire}" class="wire"/><path d="{red}" class="bad"/><path d="{orange}" class="flip"/><path d="{outline}" class="edge"/></svg>"#
        ),
        tx = t0.0,
        ty = t0.1,
        tw = t1.0 - t0.0,
        th = t1.1 - t0.1,
        wire = wire,
        red = red,
        orange = orange,
        outline = outline,
    )
}
