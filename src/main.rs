//! easyuv-lint: checks 3D assets (UVs, polycount, scale, names) for CI and artists.

mod checks;
mod config;
mod html;
mod i18n;
mod load;
mod report;
mod svg;

use config::{Config, Level, RULES};
use i18n::{rule_doc, Lang};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HELP_EN: &str = "easyuv-lint: lint 3D assets (OBJ, glTF, GLB, FBX) in seconds

USAGE
  easyuv-lint [PATHS...] [OPTIONS]

  PATHS are files or folders (searched recursively). Default: current folder.

OPTIONS
  -c, --config FILE      Configuration file (default: easyuv-lint.toml if present)
  -p, --preset NAME      game (default), mobile, web, film
  -t, --texture-size N   Texture size in px used for texel density (default 2048)
  -f, --format FORMAT    text (default), json, markdown, github
      --html FILE        Also write an HTML report with UV previews
      --json FILE        Also write the JSON report to a file
      --markdown FILE    Also append the Markdown summary to a file (e.g. $GITHUB_STEP_SUMMARY)
      --lang en|fr       Language of the messages (default: from LANG)
      --strict           Fail on warnings too
  -q, --quiet            Only show files with problems
      --list-rules       List every rule and its default
  -V, --version          Show the version
  -h, --help             Show this help

EXIT CODES
  0 no errors · 1 errors found (or warnings with --strict) · 2 invalid usage
";

const HELP_FR: &str = "easyuv-lint : vérifie tes assets 3D (OBJ, glTF, GLB, FBX) en quelques secondes

UTILISATION
  easyuv-lint [CHEMINS...] [OPTIONS]

  CHEMINS : fichiers ou dossiers (parcourus récursivement). Par défaut : le dossier courant.

OPTIONS
  -c, --config FICHIER   Fichier de configuration (par défaut : easyuv-lint.toml s'il existe)
  -p, --preset NOM       game (défaut), mobile, web, film
  -t, --texture-size N   Taille de texture en px pour la densité de texels (défaut 2048)
  -f, --format FORMAT    text (défaut), json, markdown, github
      --html FICHIER     Écrit aussi un rapport HTML avec l'aperçu des UV
      --json FICHIER     Écrit aussi le rapport JSON dans un fichier
      --markdown FICHIER Ajoute aussi le résumé Markdown à un fichier (ex. $GITHUB_STEP_SUMMARY)
      --lang en|fr       Langue des messages (défaut : d'après LANG)
      --strict           Échoue aussi sur les avertissements
  -q, --quiet            N'affiche que les fichiers à problème
      --list-rules       Liste les règles et leurs valeurs par défaut
  -V, --version          Affiche la version
  -h, --help             Affiche cette aide

CODES DE SORTIE
  0 aucune erreur · 1 erreurs trouvées (ou avertissements avec --strict) · 2 utilisation invalide
";

struct Args {
    paths: Vec<PathBuf>,
    config: Option<PathBuf>,
    preset: Option<String>,
    texture: Option<u32>,
    format: String,
    html: Option<PathBuf>,
    json: Option<PathBuf>,
    markdown: Option<PathBuf>,
    lang: Lang,
    strict: bool,
    quiet: bool,
}

enum Action {
    Run(Args),
    Help(Lang),
    Version,
    ListRules(Lang),
}

fn parse_args() -> Result<Action, String> {
    let mut lang = Lang::from_env();
    let raw: Vec<String> = std::env::args().skip(1).collect();
    // the language applies to errors and help, so read it first
    if let Some(i) = raw.iter().position(|a| a == "--lang") {
        lang = raw.get(i + 1).and_then(|v| Lang::parse(v)).ok_or("--lang expects en or fr")?;
    }
    let mut a = Args { paths: Vec::new(), config: None, preset: None, texture: None, format: "text".into(), html: None, json: None, markdown: None, lang, strict: false, quiet: false };
    let mut it = raw.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} expects a value"));
        match arg.as_str() {
            "-h" | "--help" => return Ok(Action::Help(lang)),
            "-V" | "--version" => return Ok(Action::Version),
            "--list-rules" => return Ok(Action::ListRules(lang)),
            "-c" | "--config" => a.config = Some(value("--config")?.into()),
            "-p" | "--preset" => a.preset = Some(value("--preset")?),
            "-t" | "--texture-size" => {
                a.texture = Some(value("--texture-size")?.parse().map_err(|_| "--texture-size expects a number".to_string())?);
            }
            "-f" | "--format" => {
                let f = value("--format")?;
                if !["text", "json", "markdown", "github"].contains(&f.as_str()) {
                    return Err(format!("unknown format '{f}' (text, json, markdown, github)"));
                }
                a.format = f;
            }
            "--html" => a.html = Some(value("--html")?.into()),
            "--json" => a.json = Some(value("--json")?.into()),
            "--markdown" => a.markdown = Some(value("--markdown")?.into()),
            "--lang" => {
                value("--lang")?;
            }
            "--strict" => a.strict = true,
            "-q" | "--quiet" => a.quiet = true,
            s if s.starts_with('-') && s.len() > 1 => return Err(format!("unknown option '{s}' (see --help)")),
            _ => a.paths.push(arg.into()),
        }
    }
    if a.paths.is_empty() {
        a.paths.push(".".into());
    }
    Ok(Action::Run(a))
}

/// Asset files under the given paths, sorted, skipping hidden and build folders.
fn collect(paths: &[PathBuf], ignore: &[String]) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                if !name.starts_with('.') && !["node_modules", "target", "Library", "Temp", "Intermediate", "Saved", "DerivedDataCache"].contains(&name.as_str()) {
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

fn display_path(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    s.strip_prefix("./").unwrap_or(&s).to_string()
}

fn run(a: Args) -> Result<ExitCode, String> {
    let mut cfg = match &a.config {
        Some(p) => Config::load(p)?,
        None if Path::new("easyuv-lint.toml").is_file() => Config::load(Path::new("easyuv-lint.toml"))?,
        None => Config::preset(a.preset.as_deref().unwrap_or("game"))?,
    };
    if let (Some(p), true) = (&a.preset, a.config.is_some() || Path::new("easyuv-lint.toml").is_file()) {
        // an explicit --preset replaces the file's preset but keeps its ignore list and texture size
        let keep = (cfg.ignore.clone(), cfg.texture_size);
        cfg = Config::preset(p)?;
        (cfg.ignore, cfg.texture_size) = keep;
    }
    if let Some(t) = a.texture {
        cfg.texture_size = t;
    }
    let files = collect(&a.paths, &cfg.ignore)?;
    let with_svg = a.html.is_some();
    let reports: Vec<checks::FileReport> = files
        .iter()
        .map(|p| match load::load(p) {
            Ok(asset) => checks::FileReport {
                path: display_path(p),
                format: asset.format.to_string(),
                objects: asset.objects.iter().map(|o| checks::analyze(o, &cfg, with_svg)).collect(),
                error: None,
            },
            Err(e) => checks::FileReport {
                path: display_path(p),
                format: p.extension().map(|e| e.to_string_lossy().to_uppercase()).unwrap_or_default(),
                objects: Vec::new(),
                error: Some(checks::load_error(&e)),
            },
        })
        .collect();

    let out = match a.format.as_str() {
        "json" => report::json(&reports, &cfg),
        "markdown" => report::markdown(&reports, &cfg, a.lang),
        "github" => report::github(&reports, a.lang),
        _ => {
            let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
            report::text(&reports, &cfg, a.lang, color, a.quiet)
        }
    };
    print!("{out}");
    if let Some(path) = &a.html {
        std::fs::write(path, html::render(&reports, &cfg, a.lang)).map_err(|e| format!("{}: {e}", path.display()))?;
        if a.format == "text" {
            println!("→ {}", path.display());
        }
    }
    if let Some(path) = &a.json {
        std::fs::write(path, report::json(&reports, &cfg)).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if let Some(path) = &a.markdown {
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        f.write_all(report::markdown(&reports, &cfg, a.lang).as_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let fail_at = if a.strict { Level::Warning } else { Level::Error };
    let failed = reports.iter().any(|r| r.worst() >= fail_at);
    Ok(if failed { ExitCode::from(1) } else { ExitCode::SUCCESS })
}

fn main() -> ExitCode {
    #[cfg(windows)]
    enable_utf8_console();
    match parse_args() {
        Ok(Action::Help(lang)) => {
            print!("{}", if lang == Lang::Fr { HELP_FR } else { HELP_EN });
            ExitCode::SUCCESS
        }
        Ok(Action::Version) => {
            println!("easyuv-lint {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Action::ListRules(lang)) => {
            let def = Config::preset("game").unwrap();
            for (id, ..) in RULES {
                let r = def.rule(id);
                let level = format!("{:?}", r.level).to_lowercase();
                let mut params = Vec::new();
                if let Some(m) = r.min {
                    params.push(format!("min={m}"));
                }
                if let Some(m) = r.max {
                    params.push(format!("max={m}"));
                }
                println!("{id:<18} {level:<8} {:<16} {}", params.join(" "), rule_doc(lang, id));
            }
            ExitCode::SUCCESS
        }
        Ok(Action::Run(a)) => match run(a) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("easyuv-lint: {e}");
                ExitCode::from(2)
            }
        },
        Err(e) => {
            eprintln!("easyuv-lint: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(windows)]
fn enable_utf8_console() {
    extern "system" {
        fn SetConsoleOutputCP(id: u32) -> i32;
    }
    // SAFETY: plain Win32 call with no pointers; it only changes the console code page.
    unsafe {
        SetConsoleOutputCP(65001);
    }
}
