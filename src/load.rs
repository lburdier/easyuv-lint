//! Asset loading: OBJ, glTF / GLB and FBX, keeping the UVs already in the file.
//! Positions are in world space and metres (OBJ and glTF are assumed to be in metres,
//! FBX is converted). UVs follow the OBJ convention: V points up.

use easyuv::math::V3;
use easyuv::mesh::Mesh;
use std::collections::HashMap;
use std::path::Path;

pub const EXTENSIONS: &[&str] = &["obj", "gltf", "glb", "fbx"];

/// One mesh object, with one UV per face corner (aligned with `mesh.corners`).
pub struct Object {
    pub name: String,
    pub mesh: Mesh,
    pub uv: Option<Vec<[f32; 2]>>,
    /// Faces dropped because they collapse once duplicated vertices are welded.
    pub degenerate: usize,
}

pub struct Asset {
    pub format: &'static str,
    pub objects: Vec<Object>,
}

pub fn is_asset(path: &Path) -> bool {
    path.extension().map(|e| e.to_string_lossy().to_lowercase()).is_some_and(|e| EXTENSIONS.contains(&e.as_str()))
}

pub fn load(path: &Path) -> Result<Asset, String> {
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let (format, objects) = match ext.as_str() {
        "obj" => ("OBJ", load_obj(path)?),
        "gltf" => ("glTF", load_gltf(path)?),
        "glb" => ("GLB", load_gltf(path)?),
        "fbx" => ("FBX", load_fbx(path)?),
        _ => return Err(format!("unsupported format: .{ext}")),
    };
    Ok(Asset { format, objects })
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "object".into())
}

/// Raw object before welding: faces as lists of (position index, uv).
struct RawObject {
    name: String,
    positions: Vec<V3>,
    faces: Vec<Vec<(u32, Option<[f32; 2]>)>>,
}

/// Welds coincident vertices (glTF and FBX split them at UV seams) without reordering
/// faces or corners, so corner UVs stay aligned. Faces that collapse are dropped and counted.
fn finish(raw: RawObject) -> Option<Object> {
    if raw.faces.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = (V3::new(f64::MAX, f64::MAX, f64::MAX), V3::new(f64::MIN, f64::MIN, f64::MIN));
    for p in &raw.positions {
        lo = V3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
        hi = V3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
    }
    let q = ((hi - lo).length() * 1e-6).max(1e-9);
    let mut map: HashMap<(i64, i64, i64), u32> = HashMap::new();
    let mut mesh = Mesh::new();
    let remap: Vec<u32> = raw
        .positions
        .iter()
        .map(|p| {
            let k = ((p.x / q).round() as i64, (p.y / q).round() as i64, (p.z / q).round() as i64);
            *map.entry(k).or_insert_with(|| {
                mesh.positions.push(*p);
                (mesh.positions.len() - 1) as u32
            })
        })
        .collect();
    let has_uv = raw.faces.iter().all(|f| f.iter().all(|c| c.1.is_some()));
    let mut uv = Vec::new();
    let mut degenerate = 0;
    let mut face = Vec::new();
    for f in &raw.faces {
        face.clear();
        face.extend(f.iter().map(|c| remap[c.0 as usize]));
        let mut sorted = face.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() < 3 || sorted.len() != face.len() {
            degenerate += 1;
            continue;
        }
        mesh.add_face(&face);
        if has_uv {
            uv.extend(f.iter().map(|c| c.1.unwrap()));
        }
    }
    if mesh.face_count() == 0 {
        return None;
    }
    Some(Object { name: raw.name, mesh, uv: has_uv.then_some(uv), degenerate })
}

// ---------------------------------------------------------------------------
// OBJ
// ---------------------------------------------------------------------------

fn load_obj(path: &Path) -> Result<Vec<Object>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_obj(&text, &stem(path))
}

pub fn parse_obj(text: &str, default_name: &str) -> Result<Vec<Object>, String> {
    let mut positions: Vec<V3> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    // faces use global indices; objects are split afterwards
    let mut objects: Vec<(String, Vec<Vec<(u32, Option<[f32; 2]>)>>)> = vec![(default_name.to_string(), Vec::new())];
    let has_o = text.lines().any(|l| l.trim_start().starts_with("o "));
    let resolve = |i: i64, n: usize, ln: usize| -> Result<usize, String> {
        let r = if i < 0 { n as i64 + i } else { i - 1 };
        if r < 0 || r >= n as i64 {
            Err(format!("line {}: index out of range {i}", ln + 1))
        } else {
            Ok(r as usize)
        }
    };
    for (ln, line) in text.lines().enumerate() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => {
                let c: Vec<f64> = it.take(3).map(str::parse).collect::<Result<_, _>>().map_err(|e| format!("line {}: {e}", ln + 1))?;
                if c.len() < 3 {
                    return Err(format!("line {}: incomplete vertex", ln + 1));
                }
                positions.push(V3::new(c[0], c[1], c[2]));
            }
            Some("vt") => {
                let c: Vec<f32> = it.take(2).map(str::parse).collect::<Result<_, _>>().map_err(|e| format!("line {}: {e}", ln + 1))?;
                uvs.push([c.first().copied().unwrap_or(0.0), c.get(1).copied().unwrap_or(0.0)]);
            }
            Some(tag @ ("o" | "g")) if (tag == "o") == has_o => {
                let name = it.collect::<Vec<_>>().join(" ");
                let name = if name.is_empty() { format!("{default_name}.{:03}", objects.len()) } else { name };
                if objects.last().is_some_and(|o| o.1.is_empty()) {
                    objects.last_mut().unwrap().0 = name;
                } else {
                    objects.push((name, Vec::new()));
                }
            }
            Some("f") => {
                let mut face = Vec::new();
                for tok in it {
                    let mut parts = tok.split('/');
                    let vi: i64 = parts.next().unwrap_or("").parse().map_err(|_| format!("line {}: invalid index '{tok}'", ln + 1))?;
                    let v = resolve(vi, positions.len(), ln)?;
                    let t = match parts.next() {
                        Some(s) if !s.is_empty() => {
                            let ti: i64 = s.parse().map_err(|_| format!("line {}: invalid index '{tok}'", ln + 1))?;
                            Some(uvs[resolve(ti, uvs.len(), ln)?])
                        }
                        _ => None,
                    };
                    face.push((v as u32, t));
                }
                if face.len() >= 3 {
                    objects.last_mut().unwrap().1.push(face);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for (name, faces) in objects {
        // each object keeps only the vertices it uses
        let mut local: HashMap<u32, u32> = HashMap::new();
        let mut raw = RawObject { name, positions: Vec::new(), faces: Vec::with_capacity(faces.len()) };
        for f in faces {
            let lf = f
                .into_iter()
                .map(|(v, t)| {
                    let id = *local.entry(v).or_insert_with(|| {
                        raw.positions.push(positions[v as usize]);
                        (raw.positions.len() - 1) as u32
                    });
                    (id, t)
                })
                .collect();
            raw.faces.push(lf);
        }
        out.extend(finish(raw));
    }
    if out.is_empty() {
        return Err("no faces found".into());
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// glTF / GLB
// ---------------------------------------------------------------------------

fn mat_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    // column-major matrices (glTF convention): r[column][row]
    let mut r = [[0.0f32; 4]; 4];
    for c in 0..4 {
        for l in 0..4 {
            r[c][l] = (0..4).map(|k| a[k][l] * b[c][k]).sum();
        }
    }
    r
}

fn load_gltf(path: &Path) -> Result<Vec<Object>, String> {
    let (doc, buffers, _) = gltf::import(path).map_err(|e| format!("glTF: {e}"))?;
    let identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let scene = doc.default_scene().or_else(|| doc.scenes().next()).ok_or("glTF file without a scene")?;
    let mut stack: Vec<(gltf::Node, [[f32; 4]; 4])> = scene.nodes().map(|n| (n, identity)).collect();
    let mut out = Vec::new();
    while let Some((node, parent)) = stack.pop() {
        let world = mat_mul(&parent, &node.transform().matrix());
        for child in node.children() {
            stack.push((child, world));
        }
        let Some(gmesh) = node.mesh() else { continue };
        let name = node.name().or(gmesh.name()).map(str::to_string).unwrap_or_else(|| format!("{}.{:03}", stem(path), out.len()));
        let mut raw = RawObject { name, positions: Vec::new(), faces: Vec::new() };
        for prim in gmesh.primitives() {
            if prim.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let reader = prim.reader(|b| Some(&buffers[b.index()]));
            let Some(pos) = reader.read_positions() else { continue };
            let base = raw.positions.len() as u32;
            for p in pos {
                raw.positions.push(V3::new(
                    (world[0][0] * p[0] + world[1][0] * p[1] + world[2][0] * p[2] + world[3][0]) as f64,
                    (world[0][1] * p[0] + world[1][1] * p[1] + world[2][1] * p[2] + world[3][1]) as f64,
                    (world[0][2] * p[0] + world[1][2] * p[1] + world[2][2] * p[2] + world[3][2]) as f64,
                ));
            }
            let count = raw.positions.len() as u32 - base;
            // glTF puts the UV origin at the top left: flip V to match OBJ
            let uv: Option<Vec<[f32; 2]>> = reader.read_tex_coords(0).map(|t| t.into_f32().map(|c| [c[0], 1.0 - c[1]]).collect());
            let idx: Vec<u32> = match reader.read_indices() {
                Some(i) => i.into_u32().collect(),
                None => (0..count).collect(),
            };
            for t in idx.chunks_exact(3) {
                raw.faces.push(t.iter().map(|&i| (base + i, uv.as_ref().and_then(|u| u.get(i as usize).copied()))).collect());
            }
        }
        out.extend(finish(raw));
    }
    if out.is_empty() {
        return Err("no triangle mesh found".into());
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// FBX
// ---------------------------------------------------------------------------

fn load_fbx(path: &Path) -> Result<Vec<Object>, String> {
    let opts = ufbx::LoadOpts {
        target_axes: ufbx::CoordinateAxes {
            right: ufbx::CoordinateAxis::PositiveX,
            up: ufbx::CoordinateAxis::PositiveZ,
            front: ufbx::CoordinateAxis::NegativeY,
        },
        target_unit_meters: 1.0,
        space_conversion: ufbx::SpaceConversion::ModifyGeometry,
        ..Default::default()
    };
    let scene = ufbx::load_file(&path.to_string_lossy(), opts).map_err(|e| format!("FBX: {}", AsRef::<str>::as_ref(&e.description)))?;
    let mut out = Vec::new();
    for node in scene.nodes.iter() {
        let Some(fm) = node.mesh.as_ref() else { continue };
        let name = node.element.name.to_string();
        let name = if name.is_empty() { format!("{}.{:03}", stem(path), out.len()) } else { name };
        let mut raw = RawObject { name, positions: Vec::new(), faces: Vec::new() };
        for v in fm.vertices.iter() {
            let p = ufbx::transform_position(&node.geometry_to_world, *v);
            raw.positions.push(V3::new(p.x, p.y, p.z));
        }
        // a negative scale flips the faces: restore their orientation
        let g = &node.geometry_to_world;
        let det = g.m00 * (g.m11 * g.m22 - g.m12 * g.m21) - g.m01 * (g.m10 * g.m22 - g.m12 * g.m20) + g.m02 * (g.m10 * g.m21 - g.m11 * g.m20);
        let uv = &fm.vertex_uv;
        for f in fm.faces.iter() {
            if f.num_indices < 3 {
                continue;
            }
            let mut face: Vec<(u32, Option<[f32; 2]>)> = (f.index_begin..f.index_begin + f.num_indices)
                .map(|i| {
                    let t = uv.exists.then(|| {
                        let c = uv.values[uv.indices[i as usize] as usize];
                        [c.x as f32, c.y as f32]
                    });
                    (fm.vertex_indices[i as usize], t)
                })
                .collect();
            if det < 0.0 {
                face.reverse();
            }
            raw.faces.push(face);
        }
        out.extend(finish(raw));
    }
    if out.is_empty() {
        return Err("no mesh found".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obj_keeps_uvs_and_splits_objects() {
        let src = "o A\nv 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nf 1/1 2/2 3/3 4/4\no B\nv 0 0 1\nv 1 0 1\nv 0 1 1\nf 5 6 7\n";
        let objs = parse_obj(src, "x").unwrap();
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0].name, "A");
        assert_eq!(objs[0].uv.as_ref().unwrap(), &vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        assert!(objs[1].uv.is_none());
    }

    #[test]
    fn welding_drops_collapsed_faces() {
        let src = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 1 1 0\nf 1 2 3\nf 2 3 4\n";
        let objs = parse_obj(src, "x").unwrap();
        assert_eq!(objs[0].mesh.face_count(), 1);
        assert_eq!(objs[0].degenerate, 1);
    }
}
