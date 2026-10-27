//! Output formats: terminal text, JSON, Markdown (GitHub job summary) and GitHub annotations.

use crate::checks::FileReport;
use crate::config::{Config, Level};
use crate::i18n::{count, num, t, Lang};
use std::fmt::Write;

pub struct Summary {
    pub files: usize,
    pub objects: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

impl Summary {
    pub fn of(files: &[FileReport]) -> Summary {
        let count = |l: Level| files.iter().flat_map(|f| f.findings()).filter(|x| x.level == l).count();
        Summary {
            files: files.len(),
            objects: files.iter().map(|f| f.objects.len()).sum(),
            errors: count(Level::Error),
            warnings: count(Level::Warning),
            infos: count(Level::Info),
        }
    }

    pub fn line(&self, lang: Lang) -> String {
        format!(
            "{} · {} · {} · {}",
            count(lang, self.files, "files"),
            count(lang, self.objects, "objects"),
            count(lang, self.errors, "errors"),
            count(lang, self.warnings, "warnings")
        )
    }
}

fn level_word(lang: Lang, l: Level) -> &'static str {
    match l {
        Level::Error => t(lang, "error"),
        Level::Warning => t(lang, "warning"),
        _ => t(lang, "info"),
    }
}

// ---------------------------------------------------------------------------
// Terminal
// ---------------------------------------------------------------------------

pub fn text(files: &[FileReport], cfg: &Config, lang: Lang, color: bool, quiet: bool) -> String {
    let paint = |code: &str, s: &str| if color { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() };
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {} · {} {} · {} {} px\n",
        paint("1", "easyuv-lint"),
        env!("CARGO_PKG_VERSION"),
        t(lang, "preset").to_lowercase(),
        cfg.preset,
        t(lang, "texture").to_lowercase(),
        cfg.texture_size
    );
    for f in files {
        let worst = f.worst();
        if quiet && worst < Level::Warning {
            continue;
        }
        let mark = match worst {
            Level::Error => paint("31;1", "✖"),
            Level::Warning => paint("33;1", "⚠"),
            _ => paint("32;1", "✔"),
        };
        let _ = writeln!(out, "{mark} {}  {}", paint("1", &f.path), paint("2", &format!("({}, {})", f.format, count(lang, f.objects.len(), "objects"))));
        if let Some(e) = &f.error {
            let _ = writeln!(out, "    {} {}", paint("31", level_word(lang, e.level)), e.message(lang));
        }
        for o in &f.objects {
            if quiet && o.findings.iter().all(|x| x.level < Level::Warning) {
                continue;
            }
            let mut info = format!("{} {}", num(lang, o.tris as f64, 0), t(lang, "tris"));
            match &o.uv {
                Some(u) => {
                    let _ = write!(info, " · {} · {} {}", count(lang, u.islands, "islands"), t(lang, "uv_score"), num(lang, u.score, 0));
                }
                None => {
                    let _ = write!(info, " · {}", t(lang, "no_uv"));
                }
            }
            let _ = writeln!(out, "    {}  {}", o.name, paint("2", &info));
            for x in &o.findings {
                let (code, word) = match x.level {
                    Level::Error => ("31", level_word(lang, x.level)),
                    Level::Warning => ("33", level_word(lang, x.level)),
                    _ => ("36", level_word(lang, x.level)),
                };
                let _ = writeln!(out, "      {:<14} {:<18} {}", paint(code, word), paint("2", &x.rule), x.message(lang));
            }
        }
    }
    let s = Summary::of(files);
    if files.is_empty() {
        let _ = writeln!(out, "{}", t(lang, "no_assets"));
    } else {
        let verdict = if s.errors > 0 { paint("31;1", t(lang, "failed")) } else { paint("32;1", t(lang, "passed")) };
        let _ = writeln!(out, "\n{verdict} · {}", s.line(lang));
    }
    out
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

pub fn json(files: &[FileReport], cfg: &Config) -> String {
    let s = Summary::of(files);
    let v = serde_json::json!({
        "tool": "easyuv-lint",
        "version": env!("CARGO_PKG_VERSION"),
        "preset": cfg.preset,
        "texture_size": cfg.texture_size,
        "summary": {
            "files": s.files, "objects": s.objects,
            "errors": s.errors, "warnings": s.warnings, "infos": s.infos,
            "passed": s.errors == 0,
        },
        "files": files,
    });
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Markdown (GitHub job summary, PR comment)
// ---------------------------------------------------------------------------

fn md_escape(s: &str) -> String {
    s.replace('|', "\\|").replace('<', "&lt;")
}

pub fn markdown(files: &[FileReport], cfg: &Config, lang: Lang) -> String {
    let s = Summary::of(files);
    let mut out = String::new();
    let _ = writeln!(out, "## easyuv-lint · {}\n", t(lang, "title"));
    if files.is_empty() {
        let _ = writeln!(out, "{}", t(lang, "no_assets"));
        return out;
    }
    let verdict = if s.errors > 0 { format!("✖ **{}**", t(lang, "failed")) } else { format!("✔ **{}**", t(lang, "passed")) };
    let _ = writeln!(out, "{verdict} · {} · {} `{}` · {} {} px\n", s.line(lang), t(lang, "preset"), cfg.preset, t(lang, "texture"), cfg.texture_size);
    let head = match lang {
        Lang::En => "| | File | Objects | Triangles | UV score | Findings |",
        Lang::Fr => "| | Fichier | Objets | Triangles | Score UV | Problèmes |",
    };
    let _ = writeln!(out, "{head}\n|---|---|--:|--:|--:|---|");
    for f in files {
        let mark = match f.worst() {
            Level::Error => "✖",
            Level::Warning => "⚠",
            _ => "✔",
        };
        let tris: usize = f.objects.iter().map(|o| o.tris).sum();
        let scores: Vec<f64> = f.objects.iter().filter_map(|o| o.uv.as_ref().map(|u| u.score)).collect();
        let score = if scores.is_empty() { "-".to_string() } else { num(lang, scores.iter().cloned().fold(f64::MAX, f64::min), 0) };
        let e = f.findings().filter(|x| x.level == Level::Error).count();
        let w = f.findings().filter(|x| x.level == Level::Warning).count();
        let found = if e + w == 0 { "-".to_string() } else { format!("{}, {}", count(lang, e, "errors"), count(lang, w, "warnings")) };
        let _ = writeln!(out, "| {mark} | `{}` | {} | {} | {score} | {found} |", md_escape(&f.path), f.objects.len(), num(lang, tris as f64, 0));
    }
    let with_findings: Vec<&FileReport> = files.iter().filter(|f| f.findings().any(|x| x.level >= Level::Warning)).collect();
    for f in with_findings {
        let _ = writeln!(out, "\n<details><summary><code>{}</code></summary>\n", md_escape(&f.path));
        if let Some(e) = &f.error {
            let _ = writeln!(out, "- ✖ {}", md_escape(e.message(lang)));
        }
        for o in &f.objects {
            for x in o.findings.iter().filter(|x| x.level >= Level::Warning) {
                let mark = if x.level == Level::Error { "✖" } else { "⚠" };
                let _ = writeln!(out, "- {mark} **{}** `{}`: {}", md_escape(&o.name), x.rule, md_escape(x.message(lang)));
            }
        }
        let _ = writeln!(out, "\n</details>");
    }
    let _ = writeln!(out, "\n<sub>{} [easyuv-lint](https://github.com/lburdier/easyuv-lint) {} · {}</sub>", t(lang, "generated"), env!("CARGO_PKG_VERSION"), t(lang, "powered"));
    out
}

// ---------------------------------------------------------------------------
// GitHub workflow annotations
// ---------------------------------------------------------------------------

fn gh_data(s: &str) -> String {
    s.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A")
}

fn gh_prop(s: &str) -> String {
    gh_data(s).replace(':', "%3A").replace(',', "%2C")
}

pub fn github(files: &[FileReport], lang: Lang) -> String {
    let mut out = String::new();
    for f in files {
        for (obj, x) in f.error.iter().map(|e| ("", e)).chain(f.objects.iter().flat_map(|o| o.findings.iter().map(move |x| (o.name.as_str(), x)))) {
            let cmd = match x.level {
                Level::Error => "error",
                Level::Warning => "warning",
                Level::Info => "notice",
                Level::Off => continue,
            };
            let msg = if obj.is_empty() { x.message(lang).to_string() } else { format!("{obj}: {}", x.message(lang)) };
            let _ = writeln!(out, "::{cmd} file={},title={}::{}", gh_prop(&f.path), gh_prop(&format!("easyuv-lint {}", x.rule)), gh_data(&msg));
        }
    }
    out
}
