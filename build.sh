#!/bin/bash
set -euxo pipefail
cargo build --release --target wasm32-wasi
wasi-stub -r 0 ./target/wasm32-wasi/release/jogs.wasm -o typst-package/jogs.wasm
wasm-opt typst-package/jogs.wasm -O3 --enable-bulk-memory -o typst-package/jogs.wasm
