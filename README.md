**English** · [Français](README.fr.md)

# easyuv-lint

**Catch broken 3D assets before they reach your game.** easyuv-lint checks OBJ, glTF, GLB and FBX files for UV overlaps, flipped faces, texel density, polycount, scale mistakes and naming, on your machine or on every pull request.

It is free, open source and powered by the [EasyUV](https://github.com/lburdier/EasyUV) engine, so the UV measurements are exact, not estimated on a low-resolution grid.

![HTML report with UV previews](docs/report-preview.png)

- **GitHub Action**: annotations on the faulty files, a summary in the job page, and an HTML report with UV previews.
- **CLI** for Windows, Linux and macOS: one binary, no dependencies.
- **Presets** for game, mobile, web and film, and a small `easyuv-lint.toml` to tune every rule.
- **English and French** messages and reports.

## Add it to your workflow

```yaml
# .github/workflows/assets.yml
name: 3D assets
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: lburdier/easyuv-lint@v0
        with:
          paths: Assets/Models
          preset: game
```

Inputs:

| Input | Default | Description |
|---|---|---|
| `paths` | `.` | Files or folders to check, separated by spaces or new lines |
| `preset` | | `game`, `mobile`, `web` or `film` |
| `config` | | Path to an `easyuv-lint.toml` (default: the one at the root, if present) |
| `texture-size` | | Texture size in px used for texel density |
| `strict` | `false` | Fail on warnings too |
| `lang` | `en` | `en` or `fr` |
| `report` | `easyuv-lint-report.html` | HTML report path, empty to skip it |
| `upload-report` | `true` | Upload the HTML report as an artifact |
| `version` | `latest` | A release tag, `latest`, or `source` |

Outputs: `errors`, `warnings`, `passed`.

## Use it locally

Download the binary for your system from the [releases](https://github.com/lburdier/easyuv-lint/releases/latest), or build it:

```bash
cargo install --git https://github.com/lburdier/easyuv-lint
```

```bash
easyuv-lint Assets/Models                # check a folder
easyuv-lint hero.fbx --preset mobile     # one file, mobile budget
easyuv-lint . --html report.html         # with the HTML report
easyuv-lint . --format json              # for scripts
easyuv-lint . --lang fr                  # messages in French
easyuv-lint --list-rules                 # every rule and its default
```

Exit codes: `0` no errors, `1` errors found (or warnings with `--strict`), `2` invalid usage.

## What it checks

| Rule | Default | What it catches |
|---|---|---|
| `uv.missing` | error | The object has no UVs |
| `uv.overlap` | warning, max 1 % | UV area covered twice (mirrored or stacked islands count too) |
| `uv.flipped` | warning, max 0 | Faces mirrored relative to their island |
| `uv.outside` | warning, max 1 % | UVs outside the 0-1 tile (turn off for UDIMs) |
| `uv.distortion` | warning, max 0.15 | Stretched angles and areas |
| `uv.density_spread` | warning, max 30 % | Texel density that changes between islands |
| `uv.density` | off | Texel density outside a px/m range |
| `uv.score` | warning, min 60 | Overall EasyUV score out of 100 |
| `mesh.tris` | error, max 100,000 | Triangle budget |
| `mesh.degenerate` | warning, max 0 | Collapsed or zero-area faces |
| `mesh.nonmanifold` | warning, max 0 | Edges shared by more than two faces |
| `mesh.ngons` | off | Faces with more than four sides |
| `mesh.scale` | warning, 0.01 to 100 m | Unit mistakes (a crate exported in centimetres) |
| `name.pattern` | off | Object names that break your convention |

## Presets

| Preset | Triangles | Texture | Also |
|---|---|---|---|
| `game` | 100,000 | 2048 px | The defaults above |
| `mobile` | 15,000 | 1024 px | Texel density 256 to 1024 px/m, n-gons warned |
| `web` | 50,000 | 2048 px | For three.js and other glTF viewers |
| `film` | no limit | 2048 px | More tolerant on distortion and score |

## Configuration

Put an `easyuv-lint.toml` at the root of your repository:

```toml
preset = "game"
texture_size = 2048
ignore = ["ThirdParty/"]

[rules]
"uv.overlap" = { level = "error", max = 0.5 }
"uv.outside" = "off"
"uv.density" = { min = 512, max = 1024 }
"mesh.tris" = { max = 25000 }
"name.pattern" = { pattern = "^(SM|SK)_[A-Za-z0-9_]+$" }
```

A rule takes a level (`"off"`, `"info"`, `"warning"`, `"error"`) or a table with `level`, `min`, `max` or `pattern`. Giving parameters to a rule that is off turns it on as a warning. See [`examples/easyuv-lint.toml`](examples/easyuv-lint.toml).

## Notes

- OBJ and glTF are read as metres, FBX is converted to metres.
- glTF UVs are flipped to the OBJ convention (V up) so every format is measured the same way.
- Overlaps are measured per object. Mirrored or stacked islands are often intentional: lower `uv.overlap` to `info` or raise its `max` if your pipeline relies on them.

## Contributing

Issues and pull requests are welcome, in English or French. Good first contributions: new presets (Unreal, Unity, Godot), new rules, more formats, translations.

```bash
cargo test --release
cargo run --release -- examples/assets --html report.html
```

## License

[GPL-3.0-or-later](LICENSE), like the EasyUV engine it is built on.

Made by [Lucas Burdier](https://lburdier.fr).
