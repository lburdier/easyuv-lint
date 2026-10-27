//! Measures every object and turns the configured rules into findings.

use crate::config::{Config, Level};
use crate::i18n::{num, Lang};
use crate::load::Object;
use easyuv::mesh::Mesh;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub rule: String,
    pub level: Level,
    pub value: Option<f64>,
    pub message_en: String,
    pub message_fr: String,
}

impl Finding {
    pub fn message(&self, lang: Lang) -> &str {
        match lang {
            Lang::En => &self.message_en,
            Lang::Fr => &self.message_fr,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ScorePart {
    pub key: &'static str,
    pub score: f64,
    pub weight: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct UvStats {
    pub islands: usize,
    pub score: f64,
    pub parts: Vec<ScorePart>,
    pub overlap_pct: f64,
    pub flipped: usize,
    pub outside_pct: f64,
    pub distortion: f64,
    pub density_px_m: f64,
    pub density_spread_pct: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ObjectReport {
    pub name: String,
    pub faces: usize,
    pub tris: usize,
    pub verts: usize,
    pub ngons: usize,
    pub nonmanifold: usize,
    pub degenerate: usize,
    pub dims_m: [f64; 3],
    pub uv: Option<UvStats>,
    pub findings: Vec<Finding>,
    #[serde(skip)]
    pub svg: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileReport {
    pub path: String,
    pub format: String,
    pub objects: Vec<ObjectReport>,
    pub error: Option<Finding>,
}

impl FileReport {
    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        self.error.iter().chain(self.objects.iter().flat_map(|o| o.findings.iter()))
    }
    pub fn worst(&self) -> Level {
        self.findings().map(|f| f.level).max().unwrap_or(Level::Off)
    }
}

fn finding(rule: &str, level: Level, value: Option<f64>, en: String, fr: String) -> Finding {
    Finding { rule: rule.to_string(), level, value, message_en: en, message_fr: fr }
}

pub fn load_error(message: &str) -> Finding {
    finding("file.load", Level::Error, None, format!("Cannot read the file: {message}"), format!("Fichier illisible : {message}"))
}

/// Score parts from the EasyUV engine, with stable keys.
fn part_key(name: &str) -> &'static str {
    match name {
        "Distorsion" => "distortion",
        "Chevauchements" => "overlap",
        "Faces retournées" => "flipped",
        "Remplissage" => "fill",
        "Dans la tuile" => "in_tile",
        "Densité égale" => "even_density",
        _ => "seams",
    }
}

fn nonmanifold_edges(mesh: &Mesh) -> usize {
    let mut count: HashMap<(u32, u32), u32> = HashMap::with_capacity(mesh.corners.len());
    for f in 0..mesh.face_count() {
        let vs = mesh.face(f);
        for i in 0..vs.len() {
            let (a, b) = (vs[i], vs[(i + 1) % vs.len()]);
            *count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    count.values().filter(|&&c| c > 2).count()
}

pub fn analyze(obj: &Object, cfg: &Config, with_svg: bool) -> ObjectReport {
    let mesh = &obj.mesh;
    let faces = mesh.face_count();
    let tris: usize = (0..faces).map(|f| mesh.face(f).len() - 2).sum();
    let ngons = (0..faces).filter(|&f| mesh.face(f).len() > 4).count();
    let (lo, hi) = mesh.bounds();
    let dims = [hi.x - lo.x, hi.y - lo.y, hi.z - lo.z];
    let diag2 = (hi - lo).length().powi(2).max(1e-30);
    let zero_area = (0..faces).filter(|&f| mesh.face_area(f) < diag2 * 1e-12).count();
    let degenerate = obj.degenerate + zero_area;
    let nonmanifold = nonmanifold_edges(mesh);

    let uv = obj.uv.as_ref().map(|uv| {
        let (fi, islands) = easyuv::layout::uv_islands(mesh, uv);
        let (_, seam_len) = easyuv::layout::uv_seams(mesh, uv, &fi);
        let s = easyuv::metrics::uv_score(mesh, uv, &fi, seam_len, 1024);
        UvStats {
            islands,
            score: s.total,
            parts: s.parts.iter().map(|p| ScorePart { key: part_key(p.0), score: p.1, weight: p.2 }).collect(),
            overlap_pct: s.overlap * 100.0,
            flipped: s.flipped,
            outside_pct: s.outside * 100.0,
            distortion: s.distortion,
            density_px_m: easyuv::measure_density(mesh, uv, cfg.texture_size),
            density_spread_pct: s.density_spread * 100.0,
        }
    });

    let mut out = Vec::new();
    let rule = |id: &str| cfg.rule(id);
    let max_rule = |out: &mut Vec<Finding>, id: &str, value: f64, en: &dyn Fn(&str, &str) -> String, fr: &dyn Fn(&str, &str) -> String, dec: usize| {
        let r = rule(id);
        if r.level == Level::Off {
            return;
        }
        if let Some(max) = r.max {
            if value > max + 1e-9 {
                out.push(finding(
                    id,
                    r.level,
                    Some(value),
                    en(&num(Lang::En, value, dec), &num(Lang::En, max, dec)),
                    fr(&num(Lang::Fr, value, dec), &num(Lang::Fr, max, dec)),
                ));
            }
        }
    };

    match &uv {
        None => {
            let r = rule("uv.missing");
            if r.level != Level::Off {
                out.push(finding("uv.missing", r.level, None, "No UVs: the object cannot be textured.".into(), "Pas d'UV : l'objet ne peut pas être texturé.".into()));
            }
        }
        Some(u) => {
            max_rule(
                &mut out,
                "uv.overlap",
                u.overlap_pct,
                &|v, l| format!("{v}% of the UV area is covered twice (max {l}%). Mirrored or stacked islands count too."),
                &|v, l| format!("{v} % de la surface UV est couverte deux fois (max {l} %). Les îlots en miroir ou empilés comptent aussi."),
                2,
            );
            max_rule(&mut out, "uv.flipped", u.flipped as f64, &|v, l| format!("{v} flipped faces in UV space (max {l})."), &|v, l| format!("{v} faces retournées dans les UV (max {l})."), 0);
            max_rule(
                &mut out,
                "uv.outside",
                u.outside_pct,
                &|v, l| format!("{v}% of the UV area is outside the 0-1 tile (max {l}%)."),
                &|v, l| format!("{v} % de la surface UV est hors de la tuile 0-1 (max {l} %)."),
                2,
            );
            max_rule(&mut out, "uv.distortion", u.distortion, &|v, l| format!("Mean UV distortion {v} (max {l})."), &|v, l| format!("Distorsion UV moyenne {v} (max {l})."), 3);
            if u.islands > 1 {
                max_rule(
                    &mut out,
                    "uv.density_spread",
                    u.density_spread_pct,
                    &|v, l| format!("Texel density varies by {v}% between islands (max {l}%)."),
                    &|v, l| format!("La densité de texels varie de {v} % entre les îlots (max {l} %)."),
                    1,
                );
            }
            let r = rule("uv.density");
            if r.level != Level::Off {
                let d = u.density_px_m;
                let low = r.min.is_some_and(|m| d < m);
                let high = r.max.is_some_and(|m| d > m);
                if low || high {
                    let range = |lang: Lang| match (r.min, r.max) {
                        (Some(a), Some(b)) => (num(lang, a, 0), num(lang, b, 0)),
                        (Some(a), None) => (num(lang, a, 0), "∞".into()),
                        (None, Some(b)) => ("0".into(), num(lang, b, 0)),
                        _ => ("0".into(), "∞".into()),
                    };
                    let (ea, eb) = range(Lang::En);
                    let (fa, fb) = range(Lang::Fr);
                    let tex = cfg.texture_size;
                    out.push(finding(
                        "uv.density",
                        r.level,
                        Some(d),
                        format!("Texel density {} px/m with a {tex} px texture (expected {ea} to {eb}).", num(Lang::En, d, 0)),
                        format!("Densité de texels {} px/m avec une texture de {tex} px (attendu : {fa} à {fb}).", num(Lang::Fr, d, 0)),
                    ));
                }
            }
            let r = rule("uv.score");
            if r.level != Level::Off {
                if let Some(min) = r.min {
                    if u.score < min {
                        out.push(finding(
                            "uv.score",
                            r.level,
                            Some(u.score),
                            format!("UV score {}/100 (min {}).", num(Lang::En, u.score, 0), num(Lang::En, min, 0)),
                            format!("Score UV {}/100 (min {}).", num(Lang::Fr, u.score, 0), num(Lang::Fr, min, 0)),
                        ));
                    }
                }
            }
        }
    }

    max_rule(&mut out, "mesh.tris", tris as f64, &|v, l| format!("{v} triangles (max {l})."), &|v, l| format!("{v} triangles (max {l})."), 0);
    max_rule(
        &mut out,
        "mesh.degenerate",
        degenerate as f64,
        &|v, _| format!("{v} degenerate faces (collapsed or zero area)."),
        &|v, _| format!("{v} faces dégénérées (écrasées ou d'aire nulle)."),
        0,
    );
    max_rule(
        &mut out,
        "mesh.nonmanifold",
        nonmanifold as f64,
        &|v, _| format!("{v} non-manifold edges (shared by more than 2 faces)."),
        &|v, _| format!("{v} arêtes non-manifold (partagées par plus de 2 faces)."),
        0,
    );
    max_rule(&mut out, "mesh.ngons", ngons as f64, &|v, _| format!("{v} faces with more than 4 sides."), &|v, _| format!("{v} faces de plus de 4 côtés."), 0);

    let r = rule("mesh.scale");
    if r.level != Level::Off {
        let big = dims.iter().cloned().fold(0.0, f64::max);
        if r.min.is_some_and(|m| big < m) || r.max.is_some_and(|m| big > m) {
            let (a, b) = (r.min.unwrap_or(0.0), r.max.unwrap_or(f64::INFINITY));
            out.push(finding(
                "mesh.scale",
                r.level,
                Some(big),
                format!("Largest dimension {} m (expected {} to {} m): check the export units.", num(Lang::En, big, if big >= 1.0 { 2 } else { 3 }), num(Lang::En, a, 2), num(Lang::En, b, 0)),
                format!("Plus grande dimension {} m (attendu : {} à {} m) : vérifie les unités d'export.", num(Lang::Fr, big, if big >= 1.0 { 2 } else { 3 }), num(Lang::Fr, a, 2), num(Lang::Fr, b, 0)),
            ));
        }
    }

    let r = rule("name.pattern");
    if r.level != Level::Off {
        if let Some(p) = &r.pattern {
            if regex::Regex::new(p).map(|re| !re.is_match(&obj.name)).unwrap_or(false) {
                out.push(finding(
                    "name.pattern",
                    r.level,
                    None,
                    format!("Name \"{}\" does not match {p}.", obj.name),
                    format!("Le nom « {} » ne respecte pas {p}.", obj.name),
                ));
            }
        }
    }

    out.sort_by(|a, b| b.level.cmp(&a.level));
    let svg = if with_svg { obj.uv.as_ref().map(|uv| crate::svg::uv_layout(mesh, uv)) } else { None };
    ObjectReport { name: obj.name.clone(), faces, tris, verts: mesh.positions.len(), ngons, nonmanifold, degenerate, dims_m: dims, uv, findings: out, svg }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::parse_obj;

    const QUAD: &str = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\n";

    #[test]
    fn clean_quad_passes_and_overlap_is_reported() {
        let cfg = Config::preset("game").unwrap();
        let good = parse_obj(&format!("{QUAD}vt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nf 1/1 2/2 3/3 4/4\n"), "q").unwrap();
        let r = analyze(&good[0], &cfg, false);
        assert!(r.findings.iter().all(|f| f.rule != "uv.overlap" && f.rule != "uv.flipped"), "{:?}", r.findings);
        // two triangles mapped onto the same UV area
        let bad = parse_obj(&format!("{QUAD}vt 0 0\nvt 1 0\nvt 1 1\nf 1/1 2/2 3/3\nf 1/1 3/2 4/3\n"), "q").unwrap();
        let r = analyze(&bad[0], &cfg, false);
        assert!(r.findings.iter().any(|f| f.rule == "uv.overlap"), "{:?}", r.findings);
    }

    #[test]
    fn missing_uvs_is_an_error() {
        let cfg = Config::preset("game").unwrap();
        let o = parse_obj(&format!("{QUAD}f 1 2 3 4\n"), "q").unwrap();
        let r = analyze(&o[0], &cfg, false);
        assert!(r.findings.iter().any(|f| f.rule == "uv.missing" && f.level == Level::Error));
    }
}
