## easyuv-lint 0.2.0

- **EasyUV Lint Desktop**: a Windows app with an installer (`EasyUV-Lint-Setup-x64.exe`). Drop a folder or files, see a dashboard, every problem and the UV layout of each object, edit the rules, export the HTML report or an `easyuv-lint.toml`, and find your past scans. English and French, dark and light themes.
- The command-line tool now explains itself when it is opened with a double-click instead of closing at once.
- easyuv-lint is also a Rust library (`easyuv_lint`), used by the desktop app.

## easyuv-lint 0.1.0

First release.

- Reads OBJ, glTF, GLB and FBX, keeping the UVs already in the file.
- 14 rules: missing UVs, UV overlaps, flipped faces, UVs outside the 0-1 tile, distortion, texel density and its spread, EasyUV score, triangle count, degenerate faces, non-manifold edges, n-gons, scale (unit mistakes) and naming.
- Presets: game, mobile, web, film. Configuration in `easyuv-lint.toml`.
- Outputs: terminal, JSON, Markdown summary, GitHub annotations, and an HTML report with UV previews.
- Messages and reports in English and French.
- GitHub Action with a job summary, file annotations and the HTML report as an artifact.
