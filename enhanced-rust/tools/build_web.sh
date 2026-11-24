#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown -p presskit-web
mkdir -p web/pkg
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/presskit_web.wasm
