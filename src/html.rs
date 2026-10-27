//! Self-contained HTML report in the lburdier.fr design language, English and French.

use crate::checks::FileReport;
use crate::config::{Config, Level};
use crate::i18n::{count, num, t, Lang};
use crate::report::Summary;
use std::fmt::Write;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The same content in both languages; CSS shows the active one.
fn bi(en: &str, fr: &str) -> String {
    if en == fr {
        return esc(en);
    }
    format!(r#"<span lang="en">{}</span><span lang="fr">{}</span>"#, esc(en), esc(fr))
}

fn tr(key: &str) -> String {
    bi(t(Lang::En, key), t(Lang::Fr, key))
}

fn n(v: f64, d: usize) -> String {
    bi(&num(Lang::En, v, d), &num(Lang::Fr, v, d))
}

fn level_class(l: Level) -> &'static str {
    match l {
        Level::Error => "err",
        Level::Warning => "warn",
        Level::Info => "info",
        Level::Off => "ok",
    }
}

fn level_label(l: Level) -> String {
    match l {
        Level::Error => tr("error"),
        Level::Warning => tr("warning"),
        Level::Info => tr("info"),
        Level::Off => tr("ok"),
    }
}

pub fn render(files: &[FileReport], cfg: &Config, lang: Lang) -> String {
    let s = Summary::of(files);
    let failed = s.errors > 0;
    let mut body = String::new();

    for f in files {
        let worst = f.worst();
        let _ = write!(
            body,
            r#"<section class="file reveal"><header class="file-head"><div><p class="mono">{fmt} · {objs}</p><h2>{path}</h2></div><span class="pill {cls}">{lbl}</span></header>"#,
            fmt = esc(&f.format),
            objs = bi(&count(Lang::En, f.objects.len(), "objects"), &count(Lang::Fr, f.objects.len(), "objects")),
            path = esc(&f.path),
            cls = level_class(worst),
            lbl = level_label(worst),
        );
        if let Some(e) = &f.error {
            let _ = write!(body, r#"<ul class="findings"><li class="err"><b>{}</b> {}</li></ul>"#, esc(&e.rule), bi(&e.message_en, &e.message_fr));
        }
        for o in &f.objects {
            let _ = write!(body, r#"<article class="obj">"#);
            match &o.svg {
                Some(svg) => {
                    let _ = write!(body, r#"<figure class="map">{svg}</figure>"#);
                }
                None => {
                    let _ = write!(body, r#"<figure class="map empty"><span>{}</span></figure>"#, tr("no_uv"));
                }
            }
            let _ = write!(body, r#"<div class="obj-body"><h3>{}</h3><dl class="facts">"#, esc(&o.name));
            let _ = write!(body, "<div><dt>{}</dt><dd>{}</dd></div>", tr("tris"), n(o.tris as f64, 0));
            let size = format!("{} × {} × {} m", num(Lang::En, o.dims_m[0], 2), num(Lang::En, o.dims_m[1], 2), num(Lang::En, o.dims_m[2], 2));
            let size_fr = format!("{} × {} × {} m", num(Lang::Fr, o.dims_m[0], 2), num(Lang::Fr, o.dims_m[1], 2), num(Lang::Fr, o.dims_m[2], 2));
            let _ = write!(body, "<div><dt>{}</dt><dd>{}</dd></div>", tr("size"), bi(&size, &size_fr));
            if let Some(u) = &o.uv {
                let _ = write!(body, "<div><dt>{}</dt><dd>{}</dd></div>", tr("islands"), u.islands);
                let _ = write!(body, "<div><dt>{}</dt><dd>{} px/m</dd></div>", tr("density"), n(u.density_px_m, 0));
                let _ = write!(body, r#"<div class="score"><dt>{}</dt><dd><b>{}</b><small>/100</small></dd></div>"#, tr("uv_score"), n(u.score, 0));
            }
            let _ = write!(body, "</dl>");
            if let Some(u) = &o.uv {
                let _ = write!(body, r#"<div class="parts">"#);
                for p in &u.parts {
                    let _ = write!(
                        body,
                        r#"<div class="part"><span>{}</span><i><em style="width:{:.0}%"></em></i></div>"#,
                        tr(&format!("p.{}", p.key)),
                        (p.score * 100.0).clamp(0.0, 100.0)
                    );
                }
                let _ = write!(body, "</div>");
            }
            if o.findings.is_empty() {
                let _ = write!(body, r#"<p class="clear">{}</p>"#, tr("all_clear"));
            } else {
                let _ = write!(body, r#"<ul class="findings">"#);
                for x in &o.findings {
                    let _ = write!(body, r#"<li class="{}"><b>{}</b> {}</li>"#, level_class(x.level), esc(&x.rule), bi(&x.message_en, &x.message_fr));
                }
                let _ = write!(body, "</ul>");
            }
            let _ = write!(body, "</div></article>");
        }
        let _ = write!(body, "</section>");
    }
    if files.is_empty() {
        let _ = write!(body, r#"<section class="file"><p>{}</p></section>"#, tr("no_assets"));
    }

    let verdict = if failed { tr("failed") } else { tr("passed") };
    let word = |n: usize, key: &str| bi(&count(Lang::En, n, key)[n.to_string().len() + 1..], &count(Lang::Fr, n, key)[n.to_string().len() + 1..]);
    let stats = format!(
        r#"<li><b>{}</b> {}</li><li><b>{}</b> {}</li><li class="{}"><b>{}</b> {}</li><li class="{}"><b>{}</b> {}</li>"#,
        s.files,
        word(s.files, "files"),
        s.objects,
        word(s.objects, "objects"),
        if s.errors > 0 { "err" } else { "" },
        s.errors,
        word(s.errors, "errors"),
        if s.warnings > 0 { "warn" } else { "" },
        s.warnings,
        word(s.warnings, "warnings")
    );
    TEMPLATE
        .replace("{{LANG}}", lang.code())
        .replace("{{VERSION}}", env!("CARGO_PKG_VERSION"))
        .replace("{{STATE}}", if failed { "failed" } else { "passed" })
        .replace("{{VERDICT}}", &verdict)
        .replace("{{TITLE}}", &tr("title"))
        .replace("{{TAGLINE}}", &tr("tagline"))
        .replace("{{STATS}}", &stats)
        .replace("{{PRESET_LABEL}}", &tr("preset"))
        .replace("{{PRESET}}", &esc(&cfg.preset))
        .replace("{{TEXTURE_LABEL}}", &tr("texture"))
        .replace("{{TEXTURE}}", &cfg.texture_size.to_string())
        .replace("{{LEGEND}}", &tr("legend"))
        .replace("{{GENERATED}}", &tr("generated"))
        .replace("{{POWERED}}", &tr("powered"))
        .replace("{{BODY}}", &body)
}

const TEMPLATE: &str = r##"<!doctype html>
<html lang="{{LANG}}" data-lang="{{LANG}}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>easyuv-lint report</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Archivo:wdth,wght@62..125,400..900&family=Instrument+Sans:wght@400..600&display=swap" rel="stylesheet">
<style>
:root{--bg:#efefef;--surface:#ffffff;--surface-2:#f6f6f6;--ink:#0a0a0a;--ink-70:rgba(10,10,10,.7);--ink-50:rgba(10,10,10,.5);--ink-28:rgba(10,10,10,.28);--ink-12:rgba(10,10,10,.12);--ink-6:rgba(10,10,10,.06);--accent:#0a0a0a;--on-accent:#fff;--danger:#d9473b;--warn:#c27a00;--ok:#2f8a4a;--info:#3d6fb6}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--bg:#000;--surface:#0d0d0d;--surface-2:#17171b;--ink:#fff;--ink-70:rgba(255,255,255,.7);--ink-50:rgba(255,255,255,.5);--ink-28:rgba(255,255,255,.28);--ink-12:rgba(255,255,255,.12);--ink-6:rgba(255,255,255,.06);--accent:#fff;--on-accent:#000;--danger:#f0776c;--warn:#f2b24c;--ok:#6fcf8a;--info:#86aef0}}
:root[data-theme="dark"]{--bg:#000;--surface:#0d0d0d;--surface-2:#17171b;--ink:#fff;--ink-70:rgba(255,255,255,.7);--ink-50:rgba(255,255,255,.5);--ink-28:rgba(255,255,255,.28);--ink-12:rgba(255,255,255,.12);--ink-6:rgba(255,255,255,.06);--accent:#fff;--on-accent:#000;--danger:#f0776c;--warn:#f2b24c;--ok:#6fcf8a;--info:#86aef0}
*{box-sizing:border-box}
html[data-lang="fr"] [lang="en"],html[data-lang="en"] [lang="fr"]{display:none}
body{margin:0;background:var(--bg);color:var(--ink);font:16px/1.5 "Instrument Sans",system-ui,sans-serif;-webkit-font-smoothing:antialiased}
.wrap{max-width:1200px;margin:0 auto;padding:0 24px}
.mono{font-family:ui-monospace,"SFMono-Regular",Menlo,Consolas,monospace;font-size:12px;letter-spacing:.08em;text-transform:uppercase;color:var(--ink-70);margin:0}
.top{position:sticky;top:0;z-index:5;backdrop-filter:blur(14px);background:color-mix(in srgb,var(--bg) 70%,transparent)}
.top .wrap{display:flex;align-items:center;justify-content:space-between;height:68px}
.brand{font-family:Archivo,sans-serif;font-variation-settings:'wdth' 125,'wght' 700;font-size:15px;letter-spacing:.02em;text-transform:uppercase;color:var(--ink);text-decoration:none}
.ctrl{display:flex;gap:8px}
.ctrl button{height:40px;min-width:40px;padding:0 14px;border:0;border-radius:999px;background:var(--ink-6);color:var(--ink);font:600 13px "Instrument Sans",system-ui,sans-serif;cursor:pointer}
.ctrl button:hover{background:var(--ink-12)}
.hero{padding:48px 0 32px}
.hero h1{margin:4px 0 0;padding-top:.16em;font-family:Archivo,sans-serif;font-variation-settings:'wdth' 125,'wght' 500;font-size:clamp(52px,12vw,176px);line-height:.84;letter-spacing:-.035em;text-transform:uppercase;animation:wipe .8s cubic-bezier(.2,.8,.2,1) both}
.hero[data-state="failed"] h1{color:var(--danger)}
.hero p.lead{max-width:640px;margin:22px 0 0;font-size:19px;line-height:1.4;color:var(--ink-70)}
.stats{display:flex;flex-wrap:wrap;gap:8px;margin:26px 0 0;padding:0;list-style:none}
.stats li{padding:10px 16px;border-radius:999px;background:var(--surface);font-size:14px}
.stats li b{font-weight:600}
.stats li.err b{color:var(--danger)}.stats li.warn b{color:var(--warn)}
.meta{display:flex;flex-wrap:wrap;gap:18px;margin-top:18px}
.file{background:var(--surface);border-radius:26px;padding:28px;margin:0 0 18px}
.file-head{display:flex;align-items:flex-start;justify-content:space-between;gap:16px;margin-bottom:8px}
.file-head h2{margin:6px 0 0;font-family:Archivo,sans-serif;font-variation-settings:'wdth' 110,'wght' 600;font-size:clamp(20px,2.4vw,28px);letter-spacing:-.01em;word-break:break-all}
.pill{flex:none;padding:7px 14px;border-radius:999px;font:600 12px ui-monospace,Menlo,Consolas,monospace;letter-spacing:.06em;text-transform:uppercase;background:var(--ink-6)}
.pill.err{background:var(--danger);color:#fff}.pill.warn{background:var(--warn);color:#000}.pill.ok,.pill.info{background:var(--accent);color:var(--on-accent)}
.obj{display:grid;grid-template-columns:260px 1fr;gap:26px;padding:24px 0 4px;border-top:1px solid var(--ink-12);margin-top:18px}
.map{margin:0;aspect-ratio:1;border-radius:20px;background:var(--surface-2);padding:10px;display:flex;align-items:center;justify-content:center}
.map.empty span{font:600 12px ui-monospace,Menlo,Consolas,monospace;text-transform:uppercase;letter-spacing:.08em;color:var(--ink-50)}
.uvmap{width:100%;height:100%}
.uvmap .tile{fill:none;stroke:var(--ink-28);stroke-width:3;stroke-dasharray:14 10}
.uvmap .wire{fill:none;stroke:var(--ink-12);stroke-width:1.2}
.uvmap .bad{fill:var(--danger);fill-opacity:.55;stroke:none}
.uvmap .flip{fill:var(--warn);fill-opacity:.65;stroke:none}
.uvmap .edge{fill:none;stroke:var(--ink);stroke-width:2.2;stroke-linecap:round}
.obj h3{margin:0 0 12px;font-family:Archivo,sans-serif;font-variation-settings:'wdth' 100,'wght' 600;font-size:20px}
.facts{display:flex;flex-wrap:wrap;gap:10px 26px;margin:0 0 16px}
.facts div{min-width:90px}.facts dt{font:600 11px ui-monospace,Menlo,Consolas,monospace;letter-spacing:.08em;text-transform:uppercase;color:var(--ink-50)}
.facts dd{margin:2px 0 0;font-size:15px}
.facts .score dd b{font-family:Archivo,sans-serif;font-variation-settings:'wdth' 125,'wght' 600;font-size:28px;line-height:1}
.facts .score small{color:var(--ink-50);margin-left:2px}
.parts{display:grid;grid-template-columns:repeat(auto-fill,minmax(150px,1fr));gap:8px 18px;margin:0 0 16px}
.part span{display:block;font-size:12px;color:var(--ink-70)}
.part i{display:block;height:6px;border-radius:999px;background:var(--ink-12);overflow:hidden;margin-top:4px}
.part em{display:block;height:100%;border-radius:999px;background:var(--ink)}
.findings{list-style:none;margin:0;padding:0;display:grid;gap:8px}
.findings li{padding:12px 14px 12px 16px;border-radius:14px;background:var(--surface-2);border-left:4px solid var(--ink-28);font-size:14.5px}
.findings li b{display:block;font:600 11px ui-monospace,Menlo,Consolas,monospace;letter-spacing:.06em;text-transform:uppercase;margin-bottom:2px;color:var(--ink-70)}
.findings li.err{border-left-color:var(--danger)}.findings li.warn{border-left-color:var(--warn)}.findings li.info{border-left-color:var(--info)}
.clear{margin:0;color:var(--ok);font-weight:600}
.legend{margin:6px 0 26px;font-size:13px;color:var(--ink-50)}
footer{padding:40px 0 56px;color:var(--ink-50);font-size:13px}
footer a{color:var(--ink);text-decoration:none;border-bottom:1px solid var(--ink-28)}
@keyframes wipe{from{clip-path:inset(0 100% 0 0)}to{clip-path:inset(0 0 0 0)}}
.reveal{animation:rise .6s cubic-bezier(.2,.8,.2,1) both}
@keyframes rise{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:none}}
@media (prefers-reduced-motion:reduce){.hero h1,.reveal{animation:none}}
@media (max-width:760px){.wrap{padding:0 16px}.file{padding:20px;border-radius:22px}.obj{grid-template-columns:1fr}.map{max-width:320px}}
</style>
</head>
<body>
<div class="top"><div class="wrap">
  <a class="brand" href="https://lburdier.fr" target="_blank" rel="noopener">Lucas Burdier</a>
  <div class="ctrl">
    <button id="theme" type="button" aria-label="Theme"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="12" cy="12" r="4.2"/><path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4"/></svg></button>
    <button id="lang" type="button"><span lang="en">FR</span><span lang="fr">EN</span></button>
  </div>
</div></div>
<main class="wrap">
  <section class="hero" data-state="{{STATE}}">
    <p class="mono">easyuv-lint v{{VERSION}} · {{TITLE}}</p>
    <h1>{{VERDICT}}</h1>
    <p class="lead">{{TAGLINE}}</p>
    <ul class="stats">{{STATS}}</ul>
    <div class="meta"><p class="mono">{{PRESET_LABEL}} · {{PRESET}}</p><p class="mono">{{TEXTURE_LABEL}} · {{TEXTURE}} px</p></div>
  </section>
  <p class="legend">{{LEGEND}}</p>
  {{BODY}}
</main>
<footer><div class="wrap">{{GENERATED}} <a href="https://github.com/lburdier/easyuv-lint">easyuv-lint</a> · {{POWERED}} · <a href="https://lburdier.fr">lburdier.fr</a></div></footer>
<script>
(function(){
  var root=document.documentElement;
  function get(k){try{return localStorage.getItem(k)}catch(e){return null}}
  function put(k,v){try{localStorage.setItem(k,v)}catch(e){}}
  var lang=get('eul-lang');
  if(lang==='en'||lang==='fr'){root.dataset.lang=lang;root.lang=lang}
  var theme=get('eul-theme');
  if(theme==='light'||theme==='dark')root.dataset.theme=theme;
  document.getElementById('lang').onclick=function(){
    var l=root.dataset.lang==='fr'?'en':'fr';root.dataset.lang=l;root.lang=l;put('eul-lang',l)};
  document.getElementById('theme').onclick=function(){
    var dark=root.dataset.theme?root.dataset.theme==='dark':matchMedia('(prefers-color-scheme: dark)').matches;
    var t=dark?'light':'dark';root.dataset.theme=t;put('eul-theme',t)};
})();
</script>
</body>
</html>
"##;
