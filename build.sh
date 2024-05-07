#!/bin/bash
set -euxo pipefail
# Download the wasm-minimal-protocol repo and build wasi-stub tool from source.
(
git clone https://github.com/astrale-sharp/wasm-minimal-protocol/ --depth=1
cd wasm-minimal-protocol/wasi-stub
cargo build
cp ../target/debug/wasi-stub ../..
# rm -rf wasm-minimal-protocol
)
cargo build --release --target wasm32-wasi
cargo about generate about.hbs > license.html
cp license.html typst-package/
cp README.md typst-package/
wasi-stub -r 0 ./target/wasm32-wasi/release/jogs.wasm -o typst-package/jogs.wasm
wasm-opt typst-package/jogs.wasm -O3 --enable-bulk-memory -o typst-package/jogs.wasm
