// EasyUV Lint Desktop: interface. Talks to the Rust side through Tauri commands.
'use strict';

const T = window.__TAURI__;
// Outside Tauri (browser preview) the commands are answered by a small demo stub.
const invoke = T ? T.core.invoke : (cmd, args) => window.demoInvoke(cmd, args);

const S = {
  version: '', settings: null, history: [], rules: [], presets: [],
  scan: null, view: 'dashboard', filter: 'all', query: '', file: 0,
  scanning: false, progress: null, lastPaths: [],
};

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

const L = {
  fr: {
    dashboard: 'Tableau de bord', analysis: 'Analyse', rules: 'Règles', history: 'Historique', settings: 'Paramètres',
    dash_sub: "Vue d'ensemble de tes assets 3D, de leurs problèmes et de tes analyses.",
    new_scan: 'Nouvelle analyse', open_folder: 'Ouvrir un dossier', open_files: 'Ouvrir des fichiers', rescan: 'Relancer',
    files: 'Fichiers', objects: 'Objets', errors: 'Erreurs', warnings: 'Avertissements', avg_score: 'Score UV moyen', triangles: 'Triangles',
    s_files: 'analysés en {t}', s_objects: '{n} par fichier', s_errors: 'à corriger', s_warnings: 'à vérifier', s_score: 'sur 100', s_tris: 'au total',
    worst: 'Assets à surveiller', see_detail: 'Voir le détail', uv_score: 'Score UV', budget: 'Budget',
    by_rule: 'Problèmes par règle', by_rule_sub: 'DERNIÈRE ANALYSE', recent: 'Analyses récentes', see_all: 'Tout voir',
    success: 'Succès', failed: 'Échec', ok: 'OK', err_n: '{n} err.', warn_n: '{n} avert.',
    drop_title: 'Glisse tes assets', drop_text: "Dépose un dossier ou des fichiers OBJ, glTF, GLB ou FBX n'importe où dans la fenêtre, ou choisis-les. Rien ne quitte ton ordinateur.",
    drop_now: 'Dépose pour analyser', drop_formats: 'OBJ · glTF · GLB · FBX',
    no_issue: 'Aucun problème', none_yet: "Aucune analyse pour l'instant.",
    all: 'Tout', ok_f: 'OK', search: 'Rechercher un fichier', export_html: 'Exporter le rapport HTML',
    tris: 'triangles', islands: 'îlots', size: 'Taille', density: 'Densité de texels', no_uv: "Pas d'UV", all_clear: 'Tous les tests passent.',
    legend: 'Rouge : faces qui se chevauchent. Orange : faces retournées. Pointillés : la tuile 0-1.',
    p_distortion: 'Distorsion', p_overlap: 'Chevauchements', p_flipped: 'Faces retournées', p_fill: 'Remplissage', p_in_tile: 'Dans la tuile', p_even_density: 'Densité égale', p_seams: 'Coutures',
    rules_sub: 'Ce que chaque analyse vérifie. Les changements sont enregistrés tout de suite.',
    import_toml: 'Importer un .toml', export_toml: 'Exporter easyuv-lint.toml', reset: 'Réinitialiser',
    preset: 'Préréglage', texture: 'Taille de texture', ignore: 'Dossiers ignorés', ignore_hint: 'Fragments de chemin séparés par des virgules, ex. ThirdParty/, _backup',
    pr_game: 'Jeu PC et console', pr_mobile: 'Budget mobile serré', pr_web: 'three.js et web', pr_film: 'Film et rendu',
    rule: 'Règle', level: 'Niveau', min: 'Min', max: 'Max', pattern: 'Expression',
    off: 'Désactivée', info: 'Note', warning: 'Avertissement', error: 'Erreur',
    hist_sub: 'Tes {n} dernières analyses, enregistrées sur cet ordinateur.', clear: "Vider l'historique",
    date: 'Date', path: 'Chemin', duration: 'Durée', score: 'Score',
    set_sub: "Langue, apparence et informations sur l'application.",
    language: 'Langue', theme: 'Thème', dark: 'Sombre', light: 'Clair', about: 'À propos',
    about_text: "Vérifie les assets 3D : chevauchements d'UV, faces retournées, densité de texels, polygones, échelle et noms. Propulsé par le moteur EasyUV.",
    updates: 'Mises à jour', check: 'Vérifier', up_to_date: 'Tu as la dernière version.', update_available: 'Nouvelle version disponible : {v}', download: 'Télécharger',
    ci: 'Dans ta CI', ci_text: 'Les mêmes règles tournent sur chaque pull request avec la GitHub Action.',
    ready: 'Prêt', scanning: 'Analyse {d}/{t}', last_ok: 'Tout est bon', last_err: '{n} erreur(s)', last_warn: '{n} avertissement(s)',
    saved: 'Enregistré : {p}', exported: 'Rapport exporté', no_files: "Aucun fichier 3D trouvé dans cette sélection.", copied: 'Copié',
    ago_now: "à l'instant", ago_m: 'il y a {n} min', ago_h: 'il y a {n} h', ago_d: 'il y a {n} j',
    engine: 'Moteur EasyUV', with_preset: 'préréglage {p}', obj_1: 'objet', obj_n: 'objets',
  },
  en: {
    dashboard: 'Dashboard', analysis: 'Analysis', rules: 'Rules', history: 'History', settings: 'Settings',
    dash_sub: 'An overview of your 3D assets, their problems and your scans.',
    new_scan: 'New scan', open_folder: 'Open a folder', open_files: 'Open files', rescan: 'Run again',
    files: 'Files', objects: 'Objects', errors: 'Errors', warnings: 'Warnings', avg_score: 'Average UV score', triangles: 'Triangles',
    s_files: 'checked in {t}', s_objects: '{n} per file', s_errors: 'to fix', s_warnings: 'to review', s_score: 'out of 100', s_tris: 'in total',
    worst: 'Assets to watch', see_detail: 'See details', uv_score: 'UV score', budget: 'Budget',
    by_rule: 'Problems by rule', by_rule_sub: 'LAST SCAN', recent: 'Recent scans', see_all: 'See all',
    success: 'Passed', failed: 'Failed', ok: 'OK', err_n: '{n} err.', warn_n: '{n} warn.',
    drop_title: 'Drop your assets', drop_text: 'Drop a folder or OBJ, glTF, GLB or FBX files anywhere in the window, or pick them. Nothing leaves your computer.',
    drop_now: 'Drop to check', drop_formats: 'OBJ · glTF · GLB · FBX',
    no_issue: 'No problems', none_yet: 'No scans yet.',
    all: 'All', ok_f: 'OK', search: 'Search a file', export_html: 'Export the HTML report',
    tris: 'triangles', islands: 'islands', size: 'Size', density: 'Texel density', no_uv: 'No UVs', all_clear: 'Every check passes.',
    legend: 'Red: overlapping faces. Orange: flipped faces. Dashed square: the 0-1 tile.',
    p_distortion: 'Distortion', p_overlap: 'Overlaps', p_flipped: 'Flipped faces', p_fill: 'Tile fill', p_in_tile: 'Inside 0-1', p_even_density: 'Even density', p_seams: 'Seams',
    rules_sub: 'What every scan checks. Changes are saved right away.',
    import_toml: 'Import a .toml', export_toml: 'Export easyuv-lint.toml', reset: 'Reset',
    preset: 'Preset', texture: 'Texture size', ignore: 'Ignored folders', ignore_hint: 'Path fragments separated by commas, e.g. ThirdParty/, _backup',
    pr_game: 'PC and console games', pr_mobile: 'Tight mobile budget', pr_web: 'three.js and web', pr_film: 'Film and rendering',
    rule: 'Rule', level: 'Level', min: 'Min', max: 'Max', pattern: 'Pattern',
    off: 'Off', info: 'Note', warning: 'Warning', error: 'Error',
    hist_sub: 'Your last {n} scans, stored on this computer.', clear: 'Clear history',
    date: 'Date', path: 'Path', duration: 'Duration', score: 'Score',
    set_sub: 'Language, appearance and information about the app.',
    language: 'Language', theme: 'Theme', dark: 'Dark', light: 'Light', about: 'About',
    about_text: 'Checks 3D assets: UV overlaps, flipped faces, texel density, polycount, scale and naming. Powered by the EasyUV engine.',
    updates: 'Updates', check: 'Check', up_to_date: 'You have the latest version.', update_available: 'New version available: {v}', download: 'Download',
    ci: 'In your CI', ci_text: 'The same rules run on every pull request with the GitHub Action.',
    ready: 'Ready', scanning: 'Checking {d}/{t}', last_ok: 'All good', last_err: '{n} error(s)', last_warn: '{n} warning(s)',
    saved: 'Saved: {p}', exported: 'Report exported', no_files: 'No 3D files found in this selection.', copied: 'Copied',
    ago_now: 'just now', ago_m: '{n} min ago', ago_h: '{n} h ago', ago_d: '{n} d ago',
    engine: 'EasyUV engine', with_preset: '{p} preset', obj_1: 'object', obj_n: 'objects',
  },
};

const lang = () => (S.settings && S.settings.lang) || 'fr';
const t = (k, vars) => {
  let s = (L[lang()] || L.fr)[k] ?? L.en[k] ?? k;
  if (vars) for (const [a, b] of Object.entries(vars)) s = s.replaceAll('{' + a + '}', b);
  return s;
};
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);
const num = (v, d = 0) => new Intl.NumberFormat(lang() === 'fr' ? 'fr-FR' : 'en-US', { maximumFractionDigits: d, minimumFractionDigits: d }).format(v);
const compact = (v) => new Intl.NumberFormat(lang() === 'fr' ? 'fr-FR' : 'en-US', { notation: 'compact', maximumFractionDigits: 1 }).format(v);
const ago = (ms) => {
  const m = Math.round((Date.now() - ms) / 60000);
  if (m < 1) return t('ago_now');
  if (m < 60) return t('ago_m', { n: m });
  if (m < 1440) return t('ago_h', { n: Math.round(m / 60) });
  return t('ago_d', { n: Math.round(m / 1440) });
};
const dur = (ms) => (ms < 1000 ? `${ms} ms` : `${num(ms / 1000, 1)} s`);
const objs = (n) => `${n} ${t(n === 1 ? 'obj_1' : 'obj_n')}`;
const dir = (p) => String(p).split(/[\\/]/).slice(0, -1).join('/');
const base = (p) => String(p).replace(/[\\/]+$/, '').split(/[\\/]/).pop() || p;

// ---------------------------------------------------------------------------
// Icons (24 px grid, stroked)
// ---------------------------------------------------------------------------

const P = {
  dashboard: '<rect x="3.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="3.5" y="13.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="13.5" width="7" height="7" rx="1.5"/>',
  analysis: '<path d="M4 20h16"/><path d="M7 16V9"/><path d="M12 16V5"/><path d="M17 16v-4"/>',
  rules: '<path d="M4 6h10M4 12h16M4 18h7"/><circle cx="17" cy="6" r="2"/><circle cx="14" cy="18" r="2"/>',
  history: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-1.8-.3 1.6 1.6 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1A1.6 1.6 0 0 0 9 19.4a1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.6 1.6 0 0 0 .3-1.8 1.6 1.6 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.6 1.6 0 0 0 4.6 9a1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.6 1.6 0 0 0 1.8.3H9a1.6 1.6 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 1 1.5 1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8V9a1.6 1.6 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1Z"/>',
  folder: '<path d="M3.5 7.5a2 2 0 0 1 2-2h3.6l2 2h7.4a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2Z"/>',
  file: '<path d="M14 3.5H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5Z"/><path d="M14 3.5v5h5"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  refresh: '<path d="M20 11a8 8 0 0 0-14.3-4.9L4 8"/><path d="M4 4v4h4"/><path d="M4 13a8 8 0 0 0 14.3 4.9L20 16"/><path d="M20 20v-4h-4"/>',
  download: '<path d="M12 4v11"/><path d="m7 10 5 5 5-5"/><path d="M5 20h14"/>',
  upload: '<path d="M12 20V9"/><path d="m7 14 5-5 5 5"/><path d="M5 4h14"/>',
  cube: '<path d="m12 3 8 4.5v9L12 21l-8-4.5v-9Z"/><path d="m4 7.5 8 4.5 8-4.5M12 12v9"/>',
  external: '<path d="M14 4h6v6"/><path d="M20 4 10 14"/><path d="M19 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h4"/>',
  trash: '<path d="M4 7h16M10 11v6M14 11v6"/><path d="M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3"/>',
  github: '<path d="M9 19c-4.3 1.4-4.3-2.5-6-3m12 5v-3.5c0-1 .1-1.4-.5-2 2.8-.3 5.5-1.4 5.5-6a4.6 4.6 0 0 0-1.3-3.2 4.2 4.2 0 0 0-.1-3.2s-1.1-.3-3.5 1.3a12.3 12.3 0 0 0-6.2 0C6.5 2.8 5.4 3.1 5.4 3.1a4.2 4.2 0 0 0-.1 3.2A4.6 4.6 0 0 0 4 9.5c0 4.6 2.7 5.7 5.5 6-.6.6-.6 1.2-.5 2V21"/>',
};
const icon = (name) => `<svg class="i" viewBox="0 0 24 24">${P[name] || ''}</svg>`;

// ---------------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------------

function toast(text, err) {
  const el = document.createElement('div');
  el.className = 'toast' + (err ? ' err' : '');
  el.textContent = text;
  document.getElementById('toasts').append(el);
  setTimeout(() => el.remove(), err ? 6000 : 3500);
}

function renderNav() {
  const items = [['dashboard'], ['analysis', S.scan ? S.scan.entry.files : null], ['rules'], ['history', S.history.length || null], null, ['settings']];
  document.getElementById('nav').innerHTML = `
    <div class="brand"><img src="logo.png" width="22" height="22" alt=""><b>EasyUV Lint</b></div>
    ${items.map((it) => (it ? `<button class="navitem ${S.view === it[0] ? 'on' : ''}" data-view="${it[0]}">${icon(it[0])}<span>${t(it[0])}</span>${it[1] ? `<span class="count">${it[1]}</span>` : ''}</button>` : '<div class="navsep"></div>')).join('')}
    <div class="foot"><span class="mono">${t('engine')} · v${esc(S.version)}</span></div>`;
}

function renderStatus() {
  const el = document.getElementById('status');
  if (S.scanning) {
    const p = S.progress || { done: 0, total: 0, file: '' };
    const pct = p.total ? Math.round((p.done / p.total) * 100) : 5;
    el.innerHTML = `<span class="dot run"></span><span>${t('scanning', { d: Math.min(p.done + 1, p.total || 1), t: p.total || '…' })}</span><span class="bar"><i style="width:${pct}%"></i></span><span style="color:var(--muted)">${esc(base(p.file || ''))}</span>`;
  } else if (S.scan) {
    const e = S.scan.entry;
    const [cls, txt] = e.errors ? ['err', t('last_err', { n: e.errors })] : e.warnings ? ['warn', t('last_warn', { n: e.warnings })] : ['ok', t('last_ok')];
    el.innerHTML = `<span class="dot ${cls}"></span><span>${txt}</span><span style="color:var(--muted)">· ${esc(base(e.paths[0] || ''))}</span>`;
  } else {
    el.innerHTML = `<span class="dot ok"></span><span>${t('ready')}</span><span style="color:var(--muted)">· ${t('with_preset', { p: esc(S.settings.config.preset) })}</span>`;
  }
}

function render() {
  document.documentElement.dataset.theme = S.settings.theme;
  document.documentElement.lang = lang();
  document.querySelectorAll('[data-t]').forEach((el) => (el.textContent = t(el.dataset.t)));
  renderNav();
  renderStatus();
  const view = document.getElementById('view');
  view.className = S.view === 'analysis' && S.scan ? 'split' : '';
  view.innerHTML = { dashboard: viewDashboard, analysis: viewAnalysis, rules: viewRules, history: viewHistory, settings: viewSettings }[S.view]();
  bindView();
}

function go(view) {
  S.view = view;
  render();
  document.getElementById('view').scrollTop = 0;
}

// ---------------------------------------------------------------------------
// Scans
// ---------------------------------------------------------------------------

async function runScan(paths) {
  if (!paths || !paths.length || S.scanning) return;
  S.scanning = true;
  S.progress = null;
  S.lastPaths = paths;
  renderStatus();
  try {
    const res = await invoke('scan', { paths, config: S.settings.config });
    if (!res.reports.length) {
      toast(t('no_files'), true);
    } else {
      S.scan = res;
      S.file = worstIndex(res.reports);
      S.filter = 'all';
      S.query = '';
      S.history.unshift(res.entry);
      S.history = S.history.slice(0, 60);
      S.view = S.view === 'dashboard' ? 'dashboard' : 'analysis';
    }
  } catch (e) {
    toast(String(e), true);
  } finally {
    S.scanning = false;
    render();
  }
}

async function pickAndScan(folder) {
  const paths = await invoke('pick', { folder });
  if (paths.length) runScan(paths);
}

const worst = (r) => (r.error ? 3 : Math.max(0, ...r.objects.flatMap((o) => o.findings.map((f) => ({ error: 3, warning: 2, info: 1 })[f.level] || 0))));
const worstIndex = (reports) => reports.reduce((best, r, i) => (worst(r) > worst(reports[best]) ? i : best), 0);
const countLevel = (r, lvl) => (r.error && lvl === 'error' ? 1 : 0) + r.objects.reduce((n, o) => n + o.findings.filter((f) => f.level === lvl).length, 0);
const levelClass = (w) => (w >= 3 ? 'err' : w === 2 ? 'warn' : 'ok');
const fmtExt = (r) => (r.format || '').toLowerCase();
const msg = (f) => (lang() === 'fr' ? f.message_fr : f.message_en);

function statusPills(r) {
  const e = countLevel(r, 'error');
  const w = countLevel(r, 'warning');
  if (!e && !w) return `<span class="pill ok"><span class="dot ok"></span>${t('ok')}</span>`;
  return `${e ? `<span class="pill err"><span class="dot err"></span>${t('err_n', { n: e })}</span>` : ''}${w ? `<span class="pill warn"><span class="dot warn"></span>${t('warn_n', { n: w })}</span>` : ''}`;
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

function dropCard() {
  return `<div class="drop fade">
    <div class="icon">${icon('cube')}</div>
    <div class="big">${t('drop_title')}</div>
    <p>${t('drop_text')}</p>
    <div class="actions" style="margin-top:8px"><button class="btn primary" data-act="pick-folder">${icon('folder')}${t('open_folder')}</button><button class="btn" data-act="pick-files">${icon('file')}${t('open_files')}</button></div>
    <span class="mono" style="margin-top:6px">OBJ · glTF · GLB · FBX</span>
  </div>`;
}

function viewDashboard() {
  const head = `<div class="head"><div><h1>${t('dashboard')}</h1><p>${t('dash_sub')}</p></div>
    <div class="actions">${S.lastPaths.length || S.scan ? `<button class="btn" data-act="rescan" ${S.scanning ? 'disabled' : ''}>${icon('refresh')}${t('rescan')}</button>` : ''}<button class="btn primary" data-act="pick-folder" ${S.scanning ? 'disabled' : ''}>${icon('plus')}${t('new_scan')}</button></div></div>`;
  if (!S.scan) return head + dropCard() + recentPanel(true);
  const e = S.scan.entry;
  const reps = S.scan.reports;
  const tris = e.tris;
  const stats = [
    ['files', num(e.files), t('s_files', { t: dur(e.duration_ms) }), ''],
    ['objects', num(e.objects), t('s_objects', { n: num(e.objects / Math.max(1, e.files), 1) }), ''],
    ['errors', num(e.errors), t('s_errors'), e.errors ? 'err' : ''],
    ['warnings', num(e.warnings), t('s_warnings'), e.warnings ? 'warn' : ''],
    ['avg_score', e.score == null ? '-' : num(e.score), t('s_score'), ''],
    ['triangles', compact(tris), t('s_tris'), ''],
  ];
  const budget = S.settings.config.rules.find((r) => r.id === 'mesh.tris');
  const maxTris = budget && budget.level !== 'off' && budget.max ? budget.max : null;
  const order = reps.map((r, i) => [r, i]).sort((a, b) => worst(b[0]) - worst(a[0]) || countLevel(b[0], 'warning') - countLevel(a[0], 'warning')).slice(0, 3);
  const assets = order.map(([r, i]) => {
    const scores = r.objects.filter((o) => o.uv).map((o) => o.uv.score);
    const score = scores.length ? Math.min(...scores) : null;
    const big = Math.max(0, ...r.objects.map((o) => o.tris));
    const w = worst(r);
    return `<div class="card asset fade" data-file="${i}">
      <div class="top"><div class="avatar fmt-${fmtExt(r)}">${esc((r.format || '?').slice(0, 3))}</div><div class="t"><b>${esc(base(r.path))}</b><span>${esc(dir(r.path) || r.format)}</span></div></div>
      <div class="pills">${statusPills(r)}<span class="pill">${objs(r.objects.length)}</span></div>
      <div class="meter"><span class="mono">${t('uv_score')}</span><span class="track"><i class="${score == null ? '' : score >= 75 ? 'g' : score >= 60 ? 'a' : 'c'}" style="width:${score ?? 0}%"></i></span><b>${score == null ? '-' : num(score)}</b></div>
      <div class="meter"><span class="mono">${t('budget')}</span><span class="track"><i class="${maxTris && big > maxTris ? 'c' : ''}" style="width:${maxTris ? Math.min(100, (big / maxTris) * 100) : 0}%"></i></span><b>${compact(big)}</b></div>
      <div class="foot"><span class="msg">${icon('cube')}<span>${levelClass(w) === 'ok' ? t('no_issue') : esc(msg(r.error || r.objects.flatMap((o) => o.findings)[0]) || '')}</span></span><button class="linkbtn" data-file="${i}">${t('see_detail')}</button></div>
    </div>`;
  }).join('');
  return head + `
    <div class="stats">${stats.map(([k, v, s, c]) => `<div class="card stat ${c} fade"><div class="mono">${t(k)}</div><div class="v">${v}</div><div class="s">${s}</div></div>`).join('')}</div>
    <div class="mono" style="margin:22px 0 10px">${t('worst')}</div>
    <div class="assets">${assets}</div>
    <div class="row2">${rulesPanel()}${recentPanel(false)}</div>`;
}

function rulesPanel() {
  const counts = {};
  for (const r of S.scan.reports) {
    for (const f of [r.error, ...r.objects.flatMap((o) => o.findings)].filter(Boolean)) {
      counts[f.rule] = counts[f.rule] || { e: 0, w: 0 };
      if (f.level === 'error') counts[f.rule].e++;
      else if (f.level === 'warning') counts[f.rule].w++;
    }
  }
  const rows = Object.entries(counts).filter(([, c]) => c.e + c.w).sort((a, b) => b[1].e + b[1].w - (a[1].e + a[1].w));
  const max = Math.max(1, ...rows.map(([, c]) => c.e + c.w));
  return `<div class="card panel fade"><h2>${t('by_rule')} <span class="mono">${t('by_rule_sub')}</span></h2>
    <div class="legend" style="margin-top:12px"><span><i style="background:var(--coral)"></i>${t('errors')}</span><span><i style="background:var(--amber)"></i>${t('warnings')}</span></div>
    ${rows.length ? `<div class="ruleschart">${rows.map(([rule, c]) => `<div class="rc"><code>${esc(rule)}</code><div class="bars" style="width:${Math.max(8, ((c.e + c.w) / max) * 100)}%"><i class="e" style="flex:${c.e}"></i><i class="w" style="flex:${c.w}"></i></div><b>${c.e + c.w}</b></div>`).join('')}</div>` : `<p class="clear">${t('all_clear')}</p>`}
  </div>`;
}

function recentPanel(alone) {
  const items = S.history.slice(0, 6);
  const body = items.length
    ? items.map((h, i) => `<div class="act" data-rerun="${i}"><div class="t"><b>${esc(base(h.paths[0] || ''))}${h.paths.length > 1 ? ` +${h.paths.length - 1}` : ''}</b><span>${num(h.files)} ${t('files').toLowerCase()} · ${ago(h.date)}</span></div><span class="pill ${h.errors ? 'err' : 'ok'}"><span class="dot ${h.errors ? 'err' : 'ok'}"></span>${h.errors ? t('failed') : t('success')}</span></div>`).join('')
    : `<p class="sub" style="margin:0">${t('none_yet')}</p>`;
  return `<div class="card panel fade" ${alone ? 'style="margin-top:12px"' : ''}><h2 style="justify-content:space-between">${t('recent')}${items.length ? `<button class="linkbtn" data-view="history">${t('see_all')}</button>` : ''}</h2><div class="activity" style="margin-top:8px">${body}</div></div>`;
}

// ---------------------------------------------------------------------------
// Analysis
// ---------------------------------------------------------------------------

function filtered() {
  const q = S.query.trim().toLowerCase();
  return S.scan.reports
    .map((r, i) => [r, i])
    .filter(([r]) => {
      const w = worst(r);
      if (S.filter === 'error' && w < 3) return false;
      if (S.filter === 'warning' && w !== 2) return false;
      if (S.filter === 'ok' && w >= 2) return false;
      return !q || r.path.toLowerCase().includes(q) || r.objects.some((o) => o.name.toLowerCase().includes(q));
    });
}

function viewAnalysis() {
  if (!S.scan) return `<div class="head"><div><h1>${t('analysis')}</h1></div></div>` + dropCard();
  const reps = S.scan.reports;
  const n = { all: reps.length, error: reps.filter((r) => worst(r) >= 3).length, warning: reps.filter((r) => worst(r) === 2).length, ok: reps.filter((r) => worst(r) < 2).length };
  const list = filtered();
  if (!list.some(([, i]) => i === S.file) && list.length) S.file = list[0][1];
  const r = reps[S.file];
  return `
    <div class="toolbar">
      <h1>${esc(S.scan.entry.paths.map(base).join(', '))}</h1>
      <div class="chips">${[['all', t('all')], ['error', t('errors')], ['warning', t('warnings')], ['ok', t('ok_f')]].map(([k, l]) => `<button class="chip ${S.filter === k ? 'on' : ''}" data-filter="${k}">${l}<em>${n[k]}</em></button>`).join('')}</div>
      <input class="search" id="search" placeholder="${t('search')}" value="${esc(S.query)}">
      <button class="btn" data-act="rescan" ${S.scanning ? 'disabled' : ''}>${icon('refresh')}${t('rescan')}</button>
      <button class="btn primary" data-act="export-html">${icon('download')}${t('export_html')}</button>
    </div>
    <div class="split-body">
      <div class="filelist">${list.map(([f, i]) => `<button class="fi ${i === S.file ? 'on' : ''}" data-file="${i}"><span class="dot ${levelClass(worst(f))}"></span><span class="t"><b>${esc(base(f.path))}</b><span>${esc(dir(f.path) || objs(f.objects.length))}</span></span><span class="n">${esc(f.format)}</span></button>`).join('')}</div>
      <div class="detail" id="detail">${r ? fileDetail(r) : ''}</div>
    </div>`;
}

function fileDetail(r) {
  const cards = r.objects.map((o) => {
    const facts = [
      [t('triangles'), num(o.tris)],
      [t('size'), `${num(o.dims_m[0], 2)} × ${num(o.dims_m[1], 2)} × ${num(o.dims_m[2], 2)} m`],
    ];
    if (o.uv) facts.push([t('islands'), num(o.uv.islands)], [t('density'), `${num(o.uv.density_px_m)} px/m`]);
    return `<div class="card obj fade">
      <div class="map ${o.svg ? '' : 'none'}">${o.svg || `<span>${t('no_uv')}</span>`}</div>
      <div>
        <h3>${esc(o.name)}</h3>
        <div class="facts">${facts.map(([k, v]) => `<div><span class="mono">${k}</span><span class="v">${v}</span></div>`).join('')}${o.uv ? `<div class="score"><span class="mono">${t('uv_score')}</span><span class="v">${num(o.uv.score)}<small>/100</small></span></div>` : ''}</div>
        ${o.uv ? `<div class="parts">${o.uv.parts.map((p) => `<div><span>${t('p_' + p.key)}</span><i><em style="width:${Math.round(p.score * 100)}%"></em></i></div>`).join('')}</div>` : ''}
        ${o.findings.length ? `<div class="findings">${o.findings.map((f) => `<div class="find ${f.level}"><code>${esc(f.rule)} · ${t(f.level)}</code>${esc(msg(f))}</div>`).join('')}</div>` : `<p class="clear">${t('all_clear')}</p>`}
      </div>
    </div>`;
  }).join('');
  return `<div class="dh"><div><span class="mono">${esc(r.format)} · ${objs(r.objects.length)}</span><h2>${esc(r.path)}</h2></div><div class="pills">${statusPills(r)}</div></div>
    ${r.error ? `<div class="findings" style="margin-bottom:12px"><div class="find error"><code>${esc(r.error.rule)}</code>${esc(msg(r.error))}</div></div>` : ''}
    ${r.objects.some((o) => o.svg) ? `<p class="uvlegend">${t('legend')}</p>` : ''}${cards}`;
}

// ---------------------------------------------------------------------------
// Rules
// ---------------------------------------------------------------------------

function viewRules() {
  const c = S.settings.config;
  const docs = Object.fromEntries(S.rules.map((r) => [r.id, r]));
  const sizes = [256, 512, 1024, 2048, 4096, 8192];
  const levels = ['off', 'info', 'warning', 'error'];
  const numInput = (r, k) => (['uv.missing', 'name.pattern'].includes(r.id) ? '<span class="mono">-</span>' : `<input class="input num" type="number" step="any" data-rule="${r.id}" data-k="${k}" value="${r[k] ?? ''}">`);
  return `<div class="head"><div><h1>${t('rules')}</h1><p>${t('rules_sub')}</p></div>
      <div class="actions"><button class="btn" data-act="import-toml">${icon('upload')}${t('import_toml')}</button><button class="btn" data-act="reset-rules">${icon('refresh')}${t('reset')}</button><button class="btn primary" data-act="export-toml">${icon('download')}${t('export_toml')}</button></div></div>
    <div class="grid2">
      <div class="card panel"><div class="field"><label class="mono">${t('preset')}</label><div class="seg">${S.presets.map((p) => `<button class="${c.preset === p ? 'on' : ''}" data-preset="${p}">${p}</button>`).join('')}</div><div class="hint">${t('pr_' + c.preset)}</div></div></div>
      <div class="card panel" style="display:flex;gap:22px;flex-wrap:wrap">
        <div class="field"><label class="mono">${t('texture')}</label><select class="input" id="tex">${sizes.map((s) => `<option value="${s}" ${c.texture_size === s ? 'selected' : ''}>${s} px</option>`).join('')}</select></div>
        <div class="field" style="flex:1;min-width:220px"><label class="mono">${t('ignore')}</label><input class="input" id="ignore" style="width:100%" value="${esc(c.ignore.join(', '))}"><div class="hint">${t('ignore_hint')}</div></div>
      </div>
    </div>
    <div class="card" style="overflow:hidden"><table class="rules">
      <thead><tr><th class="mono">${t('rule')}</th><th class="mono">${t('level')}</th><th class="mono">${t('min')}</th><th class="mono">${t('max')}</th></tr></thead>
      <tbody>${c.rules.map((r) => `<tr>
        <td><code>${esc(r.id)}</code><div class="d">${esc(docs[r.id] ? docs[r.id][lang()] : '')}</div>${r.id === 'name.pattern' ? `<input class="input" style="margin-top:8px;width:100%;font-family:var(--mono)" data-rule="${r.id}" data-k="pattern" placeholder="^(SM|SK)_[A-Za-z0-9_]+$" value="${esc(r.pattern || '')}">` : ''}</td>
        <td><select class="lvl ${r.level}" data-rule="${r.id}" data-k="level">${levels.map((l) => `<option value="${l}" ${r.level === l ? 'selected' : ''}>${t(l)}</option>`).join('')}</select></td>
        <td>${numInput(r, 'min')}</td><td>${numInput(r, 'max')}</td></tr>`).join('')}</tbody>
    </table></div>`;
}

let saveTimer = null;
function saveSettings(now) {
  clearTimeout(saveTimer);
  const doSave = () => invoke('save_settings', { settings: S.settings }).catch((e) => toast(String(e), true));
  if (now) doSave();
  else saveTimer = setTimeout(doSave, 400);
}

// ---------------------------------------------------------------------------
// History and settings
// ---------------------------------------------------------------------------

function viewHistory() {
  const h = S.history;
  return `<div class="head"><div><h1>${t('history')}</h1><p>${t('hist_sub', { n: h.length })}</p></div>
    <div class="actions">${h.length ? `<button class="btn ghost" data-act="clear-history">${icon('trash')}${t('clear')}</button>` : ''}</div></div>
    ${h.length ? `<div class="card" style="overflow:hidden"><table class="hist"><thead><tr>${['date', 'path', 'files', 'errors', 'warnings', 'score', 'duration'].map((k) => `<th class="mono">${t(k)}</th>`).join('')}<th></th></tr></thead><tbody>
      ${h.map((e, i) => `<tr><td>${new Date(e.date).toLocaleString(lang() === 'fr' ? 'fr-FR' : 'en-US', { dateStyle: 'medium', timeStyle: 'short' })}</td><td class="path" title="${esc(e.paths.join('\n'))}">${esc(e.paths.join(', '))}</td><td>${num(e.files)}</td><td>${e.errors ? `<span class="pill err">${e.errors}</span>` : '0'}</td><td>${e.warnings ? `<span class="pill warn">${e.warnings}</span>` : '0'}</td><td>${e.score == null ? '-' : num(e.score)}</td><td>${dur(e.duration_ms)}</td><td><button class="btn" data-rerun="${i}">${icon('refresh')}${t('rescan')}</button></td></tr>`).join('')}
    </tbody></table></div>` : `<div class="card panel"><p class="sub" style="margin:0">${t('none_yet')}</p></div>`}`;
}

function viewSettings() {
  const s = S.settings;
  return `<div class="head"><div><h1>${t('settings')}</h1><p>${t('set_sub')}</p></div></div>
    <div class="grid2">
      <div class="card panel"><label class="mono">${t('language')}</label><div style="margin-top:10px" class="seg"><button class="${s.lang === 'fr' ? 'on' : ''}" data-lang="fr">Français</button><button class="${s.lang === 'en' ? 'on' : ''}" data-lang="en">English</button></div></div>
      <div class="card panel"><label class="mono">${t('theme')}</label><div style="margin-top:10px" class="seg"><button class="${s.theme === 'dark' ? 'on' : ''}" data-theme="dark">${t('dark')}</button><button class="${s.theme === 'light' ? 'on' : ''}" data-theme="light">${t('light')}</button></div></div>
    </div>
    <div class="grid2">
      <div class="card panel"><div class="about"><img src="logo.png" alt=""><div><b>EasyUV Lint</b><div class="mono">v${esc(S.version)} · GPL-3.0</div></div></div><p class="sub" style="margin:14px 0">${t('about_text')}</p>
        <div class="actions"><button class="btn" data-open="https://github.com/lburdier/easyuv-lint">${icon('github')}GitHub</button><button class="btn" data-open="https://lburdier.github.io/easyuv-lint/">${icon('external')}Site</button><button class="btn" data-open="https://lburdier.fr">${icon('external')}lburdier.fr</button></div></div>
      <div class="card panel"><label class="mono">${t('updates')}</label><p class="sub" id="upd" style="margin:10px 0 14px">v${esc(S.version)}</p><button class="btn" data-act="check-update">${icon('refresh')}${t('check')}</button>
        <div style="margin-top:22px"><label class="mono">${t('ci')}</label><p class="sub" style="margin:8px 0 0">${t('ci_text')}</p><pre class="code">- uses: lburdier/easyuv-lint@v0
  with:
    paths: Assets/Models
    preset: ${esc(s.config.preset)}</pre></div></div>
    </div>`;
}

async function checkUpdate() {
  const el = document.getElementById('upd');
  try {
    const res = await fetch('https://api.github.com/repos/lburdier/easyuv-lint/releases/latest', { headers: { accept: 'application/vnd.github+json' } });
    const rel = await res.json();
    const latest = String(rel.tag_name || '').replace(/^v/, '');
    const newer = latest && latest.localeCompare(S.version, undefined, { numeric: true }) > 0;
    el.innerHTML = newer ? `${t('update_available', { v: esc(latest) })} <button class="linkbtn" data-open="${esc(rel.html_url)}">${t('download')}</button>` : t('up_to_date');
  } catch (e) {
    el.textContent = String(e);
  }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

function setRule(id, k, v) {
  const r = S.settings.config.rules.find((x) => x.id === id);
  if (!r) return;
  if (k === 'level') r.level = v;
  else if (k === 'pattern') r.pattern = v || null;
  else r[k] = v === '' ? null : Number(v);
  saveSettings();
}

function bindView() {
  const view = document.getElementById('view');
  const search = view.querySelector('#search');
  if (search) {
    search.oninput = () => {
      S.query = search.value;
      const pos = search.selectionStart;
      render();
      const s2 = document.getElementById('search');
      s2.focus();
      s2.setSelectionRange(pos, pos);
    };
  }
  view.querySelectorAll('[data-rule]').forEach((el) => {
    el.onchange = () => {
      setRule(el.dataset.rule, el.dataset.k, el.value);
      if (el.dataset.k === 'level') el.className = 'lvl ' + el.value;
    };
  });
  const tex = view.querySelector('#tex');
  if (tex) tex.onchange = () => { S.settings.config.texture_size = Number(tex.value); saveSettings(); };
  const ign = view.querySelector('#ignore');
  if (ign) ign.onchange = () => { S.settings.config.ignore = ign.value.split(',').map((s) => s.trim()).filter(Boolean); saveSettings(); };
}

document.addEventListener('click', async (ev) => {
  const el = ev.target.closest('[data-view],[data-act],[data-file],[data-filter],[data-rerun],[data-preset],[data-lang],[data-theme],[data-open],[data-win]');
  if (!el) return;
  const d = el.dataset;
  if (d.win && T) {
    const w = T.window.getCurrentWindow();
    if (d.win === 'min') w.minimize();
    else if (d.win === 'max') w.toggleMaximize();
    else w.close();
  } else if (d.view) go(d.view);
  else if (d.file != null && !d.act) {
    S.file = Number(d.file);
    if (S.view !== 'analysis') go('analysis');
    else render();
  } else if (d.filter) { S.filter = d.filter; render(); }
  else if (d.rerun != null) { const h = S.history[Number(d.rerun)]; if (h) runScan(h.paths); }
  else if (d.preset) {
    S.settings.config = await invoke('preset', { name: d.preset });
    saveSettings(true);
    render();
  } else if (d.lang) { S.settings.lang = d.lang; saveSettings(true); render(); }
  else if (d.theme) { S.settings.theme = d.theme; saveSettings(true); render(); }
  else if (d.open) invoke('open_path', { path: d.open });
  else if (d.act) {
    try {
      switch (d.act) {
        case 'pick-folder': await pickAndScan(true); break;
        case 'pick-files': await pickAndScan(false); break;
        case 'rescan': runScan(S.lastPaths.length ? S.lastPaths : S.scan ? S.scan.entry.paths : S.history[0] && S.history[0].paths); break;
        case 'export-html': { const p = await invoke('export_html', { lang: lang() }); if (p) toast(t('exported')); break; }
        case 'export-toml': { const p = await invoke('export_toml', { config: S.settings.config }); if (p) toast(t('saved', { p })); break; }
        case 'import-toml': { const c = await invoke('import_toml'); if (c) { S.settings.config = c; saveSettings(true); render(); } break; }
        case 'reset-rules': S.settings.config = await invoke('preset', { name: S.settings.config.preset }); saveSettings(true); render(); break;
        case 'clear-history': await invoke('clear_history'); S.history = []; render(); break;
        case 'check-update': checkUpdate(); break;
      }
    } catch (e) {
      toast(String(e), true);
    }
  }
});

document.addEventListener('keydown', (ev) => {
  if (ev.ctrlKey && ev.key.toLowerCase() === 'o') { ev.preventDefault(); pickAndScan(true); }
  if (ev.key === 'F5') { ev.preventDefault(); if (S.lastPaths.length) runScan(S.lastPaths); }
});

async function start() {
  const b = await invoke('boot');
  Object.assign(S, { version: b.version, settings: b.settings, history: b.history, rules: b.rules, presets: b.presets });
  render();
  if (T) {
    const overlay = document.getElementById('overlay');
    T.event.listen('scan-progress', (e) => { S.progress = e.payload; renderStatus(); });
    T.event.listen('tauri://drag-enter', () => overlay.classList.add('on'));
    T.event.listen('tauri://drag-over', () => overlay.classList.add('on'));
    T.event.listen('tauri://drag-leave', () => overlay.classList.remove('on'));
    T.event.listen('tauri://drag-drop', (e) => { overlay.classList.remove('on'); runScan((e.payload && e.payload.paths) || []); });
  }
  if (b.open && b.open.length) runScan(b.open);
}

start();
