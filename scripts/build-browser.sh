#!/usr/bin/env bash
set -euo pipefail

READER_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$READER_ROOT"
export FERROUI_BROWSER_WORKSPACE="$READER_ROOT"
# Thin LTO with more codegen units bounds peak LLVM memory on ordinary runners.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
export CARGO_PROFILE_BROWSER_CODEGEN_UNITS="${CARGO_PROFILE_BROWSER_CODEGEN_UNITS:-16}"

READER_TARGET="${CARGO_TARGET_DIR:-$READER_ROOT/target}"
READER_MODE="browser"
READER_FEATURES=""
READER_SITE=""
READER_ARGUMENTS=("$@")
while [ "$#" -gt 0 ]; do
  case "$1" in
    --out|--features)
      [ "$#" -ge 2 ] || { echo "$1 requires a value" >&2; exit 2; }
      if [ "$1" = "--out" ]; then READER_SITE="$2"; else READER_FEATURES="$2"; fi
      shift
      ;;
    --threads) READER_MODE="browser-threads";;
    --both) READER_MODE="browser-both";;
  esac
  shift
done
if [ -z "$READER_SITE" ]; then
  READER_SITE="$READER_TARGET/$READER_MODE${READER_FEATURES:+-${READER_FEATURES//,/-}}/reader-browser"
fi

FerroUI/scripts/build-browser.sh reader-browser ${READER_ARGUMENTS[@]+"${READER_ARGUMENTS[@]}"}
mkdir -p "$READER_SITE/fonts"
cp crates/reader-fonts/assets/fonts/* "$READER_SITE/fonts/"
# A small independent WASM module does HTML5/Readability parsing in Web Workers.
# It is loaded on demand and keeps the UI module free of the extraction engine.
rustup target add wasm32-unknown-unknown
cargo build --locked --target wasm32-unknown-unknown --profile browser -p reader-extract
wasm-bindgen --target web --no-typescript --out-dir "$READER_SITE/extractor" \
  "$READER_TARGET/wasm32-unknown-unknown/browser/reader_extract.wasm"
node scripts/prepare-site.mjs "$READER_SITE"
