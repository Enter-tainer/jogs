#!/bin/bash
set -euxo pipefail
cargo build --release --target wasm32-wasip1
cargo about generate about.hbs > license.html
cp license.html typst-package/
cp README.md typst-package/
wasi-stub -r 0 ./target/wasm32-wasip1/release/jogs.wasm -o typst-package/jogs.wasm
wasm-opt typst-package/jogs.wasm -O3 --enable-bulk-memory -o typst-package/jogs.wasm
