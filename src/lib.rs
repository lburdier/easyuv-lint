//! easyuv-lint: checks 3D assets (UVs, polycount, scale, names). Used by the command line
//! tool and the desktop app.

pub mod checks;
pub mod config;
pub mod html;
pub mod i18n;
pub mod load;
pub mod report;
pub mod svg;

use config::Config;
use std::path::{Path, PathBuf};

/// Folders never searched: hidden ones, build output and engine caches.
const SKIPPED_DIRS: &[&str] = &["node_modules", "target", "Library", "Temp", "Intermediate", "Saved", "DerivedDataCache"];

/// Asset files under the given paths, sorted, skipping hidden and build folders.
pub fn collect(paths: &[PathBuf], ignore: &[String]) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                if !name.starts_with('.') && !SKIPPED_DIRS.contains(&name.as_str()) {
                    walk(&p, out);
                }
            } else if load::is_asset(&p) {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            walk(p, &mut out);
        } else if p.is_file() {
            out.push(p.clone());
        } else {
            return Err(format!("not found: {}", p.display()));
        }
    }
    let norm = |p: &Path| p.to_string_lossy().replace('\\', "/");
    out.retain(|p| !ignore.iter().any(|i| norm(p).contains(i.as_str())));
    out.sort();
    out.dedup();
    Ok(out)
}

pub fn display_path(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    s.strip_prefix("./").unwrap_or(&s).to_string()
}

/// Loads one file and runs every rule on each of its objects.
pub fn check_file(path: &Path, cfg: &Config, with_svg: bool) -> checks::FileReport {
    match load::load(path) {
        Ok(asset) => checks::FileReport {
            path: display_path(path),
            format: asset.format.to_string(),
            objects: asset.objects.iter().map(|o| checks::analyze(o, cfg, with_svg)).collect(),
            error: None,
        },
        Err(e) => checks::FileReport {
            path: display_path(path),
            format: path.extension().map(|e| e.to_string_lossy().to_uppercase()).unwrap_or_default(),
            objects: Vec::new(),
            error: Some(checks::load_error(&e)),
        },
    }
}
