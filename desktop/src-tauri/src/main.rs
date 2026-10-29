// EasyUV Lint Desktop: the easyuv-lint checks behind a graphical interface.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use easyuv_lint::checks::FileReport;
use easyuv_lint::config::{Config, Settings, RULES};
use easyuv_lint::i18n::{rule_doc, Lang};
use easyuv_lint::report::Summary;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager, State};

const HISTORY_MAX: usize = 60;

#[derive(Clone, Serialize, Deserialize)]
struct AppSettings {
    lang: String,
    theme: String,
    config: Settings,
}

impl Default for AppSettings {
    fn default() -> Self {
        let lang = if Lang::from_env() == Lang::Fr || system_is_french() { "fr" } else { "en" };
        AppSettings { lang: lang.into(), theme: "dark".into(), config: Config::preset("game").unwrap().to_settings() }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct HistoryEntry {
    id: u64,
    date: u64,
    paths: Vec<String>,
    files: usize,
    objects: usize,
    errors: usize,
    warnings: usize,
    tris: usize,
    score: Option<f64>,
    duration_ms: u64,
    preset: String,
}

#[derive(Clone, Serialize)]
struct ScanResult {
    entry: HistoryEntry,
    reports: Vec<FileReport>,
}

#[derive(Default)]
struct AppState {
    last: Mutex<Option<(ScanResult, Config)>>,
}

#[derive(Clone, Serialize)]
struct Progress {
    done: usize,
    total: usize,
    file: String,
}

#[derive(Serialize)]
struct RuleDoc {
    id: &'static str,
    en: &'static str,
    fr: &'static str,
}

/// Windows UI language, when LANG is not set (the usual case on Windows).
fn system_is_french() -> bool {
    #[cfg(windows)]
    {
        extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        // SAFETY: no arguments, returns a LANGID.
        let id = unsafe { GetUserDefaultUILanguage() };
        id & 0x3ff == 0x0c
    }
    #[cfg(not(windows))]
    false
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn data_file(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(name))
}

fn read_json<T: for<'de> Deserialize<'de>>(app: &tauri::AppHandle, name: &str) -> Option<T> {
    let text = std::fs::read_to_string(data_file(app, name).ok()?).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_json<T: Serialize>(app: &tauri::AppHandle, name: &str, value: &T) -> Result<(), String> {
    let text = serde_json::to_string(value).map_err(|e| e.to_string())?;
    std::fs::write(data_file(app, name)?, text).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct Boot {
    version: &'static str,
    settings: AppSettings,
    history: Vec<HistoryEntry>,
    rules: Vec<RuleDoc>,
    presets: Vec<&'static str>,
    /// Files or folders given on the command line ("Open with", drop on the icon).
    open: Vec<String>,
}

#[tauri::command]
fn boot(app: tauri::AppHandle) -> Boot {
    let settings: AppSettings = read_json(&app, "settings.json").unwrap_or_default();
    // a settings file from another version may lack rules: rebuild it from its preset
    let settings = match Config::from_settings(&settings.config) {
        Ok(cfg) => AppSettings { config: cfg.to_settings(), ..settings },
        Err(_) => AppSettings { config: Config::preset("game").unwrap().to_settings(), ..settings },
    };
    Boot {
        version: env!("CARGO_PKG_VERSION"),
        settings,
        history: read_json(&app, "history.json").unwrap_or_default(),
        rules: RULES.iter().map(|(id, ..)| RuleDoc { id, en: rule_doc(Lang::En, id), fr: rule_doc(Lang::Fr, id) }).collect(),
        presets: easyuv_lint::config::PRESETS.to_vec(),
        open: std::env::args().skip(1).filter(|a| std::path::Path::new(a).exists()).collect(),
    }
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    Config::from_settings(&settings.config)?;
    write_json(&app, "settings.json", &settings)
}

#[tauri::command]
fn preset(name: String) -> Result<Settings, String> {
    Ok(Config::preset(&name)?.to_settings())
}

#[tauri::command]
async fn pick(folder: bool) -> Vec<String> {
    let dialog = rfd::AsyncFileDialog::new();
    let picked = if folder {
        dialog.pick_folders().await
    } else {
        dialog.add_filter("3D", easyuv_lint::load::EXTENSIONS).pick_files().await
    };
    picked.unwrap_or_default().into_iter().map(|f| f.path().to_string_lossy().to_string()).collect()
}

#[tauri::command]
async fn scan(app: tauri::AppHandle, paths: Vec<String>, config: Settings) -> Result<ScanResult, String> {
    let cfg = Config::from_settings(&config)?;
    let app2 = app.clone();
    let cfg2 = cfg.clone();
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<ScanResult, String> {
        let start = Instant::now();
        let roots: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let files = easyuv_lint::collect(&roots, &cfg2.ignore)?;
        let total = files.len();
        let mut reports = Vec::with_capacity(total);
        for (i, f) in files.iter().enumerate() {
            let _ = app2.emit("scan-progress", Progress { done: i, total, file: easyuv_lint::display_path(f) });
            let mut r = easyuv_lint::check_file(f, &cfg2, true);
            // show paths relative to the scanned folder when there is a single one
            if let [root] = roots.as_slice() {
                if root.is_dir() {
                    if let Ok(rel) = f.strip_prefix(root) {
                        r.path = easyuv_lint::display_path(rel);
                    }
                }
            }
            reports.push(r);
        }
        let _ = app2.emit("scan-progress", Progress { done: total, total, file: String::new() });
        let s = Summary::of(&reports);
        let scores: Vec<f64> = reports.iter().flat_map(|r| r.objects.iter().filter_map(|o| o.uv.as_ref().map(|u| u.score))).collect();
        let entry = HistoryEntry {
            id: now_ms(),
            date: now_ms(),
            paths,
            files: s.files,
            objects: s.objects,
            errors: s.errors,
            warnings: s.warnings,
            tris: reports.iter().flat_map(|r| r.objects.iter().map(|o| o.tris)).sum(),
            score: (!scores.is_empty()).then(|| scores.iter().sum::<f64>() / scores.len() as f64),
            duration_ms: start.elapsed().as_millis() as u64,
            preset: cfg2.preset.clone(),
        };
        Ok(ScanResult { entry, reports })
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut history: Vec<HistoryEntry> = read_json(&app, "history.json").unwrap_or_default();
    history.insert(0, result.entry.clone());
    history.truncate(HISTORY_MAX);
    write_json(&app, "history.json", &history)?;
    *app.state::<AppState>().last.lock().unwrap() = Some((result.clone(), cfg));
    Ok(result)
}

#[tauri::command]
fn clear_history(app: tauri::AppHandle) -> Result<(), String> {
    write_json(&app, "history.json", &Vec::<HistoryEntry>::new())
}

#[tauri::command]
async fn export_html(state: State<'_, AppState>, lang: String) -> Result<Option<String>, String> {
    let (html, name) = {
        let last = state.last.lock().unwrap();
        let (scan, cfg) = last.as_ref().ok_or("no scan yet")?;
        let lang = Lang::parse(&lang).unwrap_or(Lang::En);
        (easyuv_lint::html::render(&scan.reports, cfg, lang), "easyuv-lint-report.html")
    };
    let Some(file) = rfd::AsyncFileDialog::new().set_file_name(name).add_filter("HTML", &["html"]).save_file().await else {
        return Ok(None);
    };
    std::fs::write(file.path(), html).map_err(|e| e.to_string())?;
    let path = file.path().to_string_lossy().to_string();
    open_path(path.clone());
    Ok(Some(path))
}

#[tauri::command]
async fn export_toml(config: Settings) -> Result<Option<String>, String> {
    let cfg = Config::from_settings(&config)?;
    let Some(file) = rfd::AsyncFileDialog::new().set_file_name("easyuv-lint.toml").add_filter("TOML", &["toml"]).save_file().await else {
        return Ok(None);
    };
    std::fs::write(file.path(), cfg.to_toml()).map_err(|e| e.to_string())?;
    Ok(Some(file.path().to_string_lossy().to_string()))
}

#[tauri::command]
async fn import_toml() -> Result<Option<Settings>, String> {
    let Some(file) = rfd::AsyncFileDialog::new().add_filter("TOML", &["toml"]).pick_file().await else {
        return Ok(None);
    };
    Ok(Some(Config::load(file.path())?.to_settings()))
}

/// Opens a file, folder or URL with the system default application.
#[tauri::command]
fn open_path(path: String) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", &path]).creation_flags(0x0800_0000).spawn();
    }
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&path).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![boot, save_settings, preset, pick, scan, clear_history, export_html, export_toml, import_toml, open_path])
        .run(tauri::generate_context!())
        .expect("error while running EasyUV Lint");
}
