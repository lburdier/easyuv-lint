// Browser preview of the desktop interface, with a recorded scan instead of the Rust side.
(function () {
  const rules = ['uv.missing','uv.overlap','uv.flipped','uv.outside','uv.distortion','uv.density_spread','uv.density','uv.score','mesh.tris','mesh.degenerate','mesh.nonmanifold','mesh.ngons','mesh.scale','name.pattern'];
  const lv = {'uv.missing':'error','uv.density':'off','mesh.tris':'error','mesh.ngons':'off','name.pattern':'off'};
  const cfg = { preset: 'game', texture_size: 2048, ignore: [], rules: rules.map((id) => ({ id, level: lv[id] || 'warning', min: id === 'uv.score' ? 60 : id === 'mesh.scale' ? 0.01 : null, max: id === 'mesh.tris' ? 100000 : id === 'uv.overlap' ? 1 : null, pattern: null })) };
  const now = Date.now();
  const scan = window.DEMO_SCAN; scan.entry.date = now - 60000;
  const hist = [scan.entry, { ...scan.entry, id: 2, date: now - 3 * 3600e3, paths: ['C:/Projets/MonJeu/Assets/Props'], errors: 0, warnings: 2, files: 12 }, { ...scan.entry, id: 3, date: now - 26 * 3600e3, paths: ['C:/Projets/Diorama/export'], errors: 0, warnings: 0, files: 4 }];
  const view = new URLSearchParams(location.search).get('view');
  window.demoInvoke = async (cmd) => {
    if (cmd === 'boot') {
      setTimeout(() => { if (view !== 'empty') { S.scan = scan; S.lastPaths = scan.entry.paths; S.file = 1; } if (view && view !== 'empty') S.view = view; render(); }, 0);
      return { version: '0.2.0', settings: { lang: new URLSearchParams(location.search).get('lang') || 'fr', theme: new URLSearchParams(location.search).get('theme') || 'dark', config: cfg }, history: hist, rules: rules.map((id) => ({ id, en: id, fr: id })), presets: ['game', 'mobile', 'web', 'film'] };
    }
    if (cmd === 'preset') return cfg;
    return null;
  };
})();
