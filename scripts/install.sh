#!/usr/bin/env bash
# Installs easyuv-lint for the GitHub Action: a release binary, or a build from source.
set -euo pipefail

version="${EUL_VERSION:-latest}"
repo="lburdier/easyuv-lint"
dest="${RUNNER_TEMP:-/tmp}/easyuv-lint/bin"
mkdir -p "$dest"

if [ "$version" = "source" ]; then
  cargo build --release --locked --manifest-path "$GITHUB_ACTION_PATH/Cargo.toml"
  cp "$GITHUB_ACTION_PATH"/target/release/easyuv-lint* "$dest/"
  echo "$dest" >> "$GITHUB_PATH"
  exit 0
fi

case "${RUNNER_OS:-$(uname -s)}-${RUNNER_ARCH:-$(uname -m)}" in
  Linux-X64|Linux-x86_64) target="x86_64-unknown-linux-gnu"; ext="tar.gz" ;;
  Linux-ARM64|Linux-aarch64) target="aarch64-unknown-linux-gnu"; ext="tar.gz" ;;
  macOS-ARM64|Darwin-arm64) target="aarch64-apple-darwin"; ext="tar.gz" ;;
  macOS-X64|Darwin-x86_64) target="x86_64-apple-darwin"; ext="tar.gz" ;;
  Windows-X64|*MINGW*|*MSYS*) target="x86_64-pc-windows-msvc"; ext="zip" ;;
  *) echo "::error::easyuv-lint has no build for ${RUNNER_OS:-?} ${RUNNER_ARCH:-?}. Use version: source."; exit 1 ;;
esac

asset="easyuv-lint-$target.$ext"
if [ "$version" = "latest" ]; then
  url="https://github.com/$repo/releases/latest/download/$asset"
else
  url="https://github.com/$repo/releases/download/$version/$asset"
fi

echo "Downloading $url"
tmp="$(mktemp -d)"
curl -fsSL --retry 3 -o "$tmp/$asset" "$url"
if [ "$ext" = "zip" ]; then
  unzip -q -o "$tmp/$asset" -d "$dest"
else
  tar -xzf "$tmp/$asset" -C "$dest"
fi
chmod +x "$dest"/easyuv-lint* 2>/dev/null || true
echo "$dest" >> "$GITHUB_PATH"
"$dest"/easyuv-lint* --version
