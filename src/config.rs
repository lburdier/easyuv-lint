//! Rules, presets and the `easyuv-lint.toml` configuration file.

use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Off,
    Info,
    Warning,
    Error,
}

impl Level {
    fn parse(s: &str) -> Result<Level, String> {
        match s.to_ascii_lowercase().as_str() {
            "off" | "none" | "false" => Ok(Level::Off),
            "info" => Ok(Level::Info),
            "warning" | "warn" => Ok(Level::Warning),
            "error" | "err" => Ok(Level::Error),
            _ => Err(format!("unknown level '{s}' (use off, info, warning or error)")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub level: Level,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub pattern: Option<String>,
}

/// Every rule, in display order: id, default level, default min, default max.
pub const RULES: &[(&str, Level, Option<f64>, Option<f64>)] = &[
    ("uv.missing", Level::Error, None, None),
    ("uv.overlap", Level::Warning, None, Some(1.0)),
    ("uv.flipped", Level::Warning, None, Some(0.0)),
    ("uv.outside", Level::Warning, None, Some(1.0)),
    ("uv.distortion", Level::Warning, None, Some(0.15)),
    ("uv.density_spread", Level::Warning, None, Some(30.0)),
    ("uv.density", Level::Off, None, None),
    ("uv.score", Level::Warning, Some(60.0), None),
    ("mesh.tris", Level::Error, None, Some(100_000.0)),
    ("mesh.degenerate", Level::Warning, None, Some(0.0)),
    ("mesh.nonmanifold", Level::Warning, None, Some(0.0)),
    ("mesh.ngons", Level::Off, None, Some(0.0)),
    ("mesh.scale", Level::Warning, Some(0.01), Some(100.0)),
    ("name.pattern", Level::Off, None, None),
];

pub const PRESETS: &[&str] = &["game", "mobile", "web", "film"];

#[derive(Clone, Debug)]
pub struct Config {
    pub preset: String,
    pub texture_size: u32,
    pub rules: BTreeMap<&'static str, Rule>,
    /// Paths containing one of these fragments are skipped.
    pub ignore: Vec<String>,
}

impl Config {
    pub fn preset(name: &str) -> Result<Config, String> {
        let mut rules: BTreeMap<&'static str, Rule> =
            RULES.iter().map(|&(id, level, min, max)| (id, Rule { level, min, max, pattern: None })).collect();
        let mut texture_size = 2048;
        let mut set = |id: &str, level: Option<Level>, min: Option<f64>, max: Option<f64>| {
            let r = rules.get_mut(id).unwrap();
            if let Some(l) = level {
                r.level = l;
            }
            if min.is_some() {
                r.min = min;
            }
            if max.is_some() {
                r.max = max;
            }
        };
        match name {
            "game" => {}
            "mobile" => {
                texture_size = 1024;
                set("mesh.tris", None, None, Some(15_000.0));
                set("uv.density", Some(Level::Warning), Some(256.0), Some(1024.0));
                set("mesh.ngons", Some(Level::Warning), None, None);
            }
            "web" => {
                set("mesh.tris", None, None, Some(50_000.0));
            }
            "film" => {
                set("mesh.tris", Some(Level::Off), None, None);
                set("uv.score", None, Some(50.0), None);
                set("uv.distortion", None, None, Some(0.25));
            }
            _ => return Err(format!("unknown preset '{name}' (use {})", PRESETS.join(", "))),
        }
        Ok(Config { preset: name.to_string(), texture_size, rules, ignore: Vec::new() })
    }

    /// Reads a TOML configuration on top of its preset.
    pub fn from_toml(text: &str) -> Result<Config, String> {
        let v: toml::Table = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
        let preset = v.get("preset").and_then(|p| p.as_str()).unwrap_or("game");
        let mut cfg = Config::preset(preset)?;
        for (key, value) in &v {
            match key.as_str() {
                "preset" => {}
                "texture_size" => {
                    cfg.texture_size = value.as_integer().filter(|&n| n > 0).ok_or("texture_size must be a positive integer")? as u32;
                }
                "ignore" => {
                    cfg.ignore = value
                        .as_array()
                        .ok_or("ignore must be a list of path fragments")?
                        .iter()
                        .map(|s| s.as_str().map(str::to_string).ok_or("ignore must contain strings"))
                        .collect::<Result<_, _>>()?;
                }
                "rules" => {
                    let table = value.as_table().ok_or("[rules] must be a table")?;
                    for (id, rv) in table {
                        let rule = cfg.rules.get_mut(id.as_str()).ok_or_else(|| format!("unknown rule '{id}' (run --list-rules)"))?;
                        apply(rule, rv).map_err(|e| format!("rule '{id}': {e}"))?;
                    }
                }
                other => return Err(format!("unknown key '{other}'")),
            }
        }
        Ok(cfg)
    }

    pub fn load(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Config::from_toml(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn rule(&self, id: &str) -> &Rule {
        &self.rules[id]
    }
}

fn apply(rule: &mut Rule, v: &toml::Value) -> Result<(), String> {
    let num = |x: &toml::Value| x.as_float().or_else(|| x.as_integer().map(|i| i as f64)).ok_or("expected a number".to_string());
    match v {
        toml::Value::String(s) => rule.level = Level::parse(s)?,
        toml::Value::Boolean(b) => rule.level = if *b { rule.level.max(Level::Warning) } else { Level::Off },
        toml::Value::Table(t) => {
            // giving parameters to a rule that is off by default turns it on as a warning
            if rule.level == Level::Off && !t.contains_key("level") {
                rule.level = Level::Warning;
            }
            for (k, x) in t {
                match k.as_str() {
                    "level" => rule.level = Level::parse(x.as_str().ok_or("level must be a string")?)?,
                    "min" => rule.min = Some(num(x)?),
                    "max" => rule.max = Some(num(x)?),
                    "pattern" => {
                        let p = x.as_str().ok_or("pattern must be a string")?;
                        regex::Regex::new(p).map_err(|e| e.to_string())?;
                        rule.pattern = Some(p.to_string());
                    }
                    other => return Err(format!("unknown option '{other}'")),
                }
            }
        }
        _ => return Err("expected a level (\"warning\") or a table ({ max = 10 })".into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_overrides_preset() {
        let cfg = Config::from_toml(
            "preset = \"mobile\"\ntexture_size = 512\n[rules]\n\"uv.overlap\" = \"error\"\n\"mesh.tris\" = { max = 8000 }\n\"name.pattern\" = { pattern = \"^SM_\" }\n",
        )
        .unwrap();
        assert_eq!(cfg.texture_size, 512);
        assert_eq!(cfg.rule("uv.overlap").level, Level::Error);
        assert_eq!(cfg.rule("mesh.tris").max, Some(8000.0));
        assert_eq!(cfg.rule("name.pattern").level, Level::Warning);
        assert!(Config::from_toml("[rules]\n\"nope\" = \"off\"\n").is_err());
    }
}
