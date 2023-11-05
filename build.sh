#!/bin/bash

cargo build --release --target wasm32-wasi
wasi-stub ./target/wasm32-wasi/release/jogs.wasm -o typst-package/jogs.wasm
wasm-opt typst-package/jogs.wasm -Oz --enable-bulk-memory -o typst-package/jogs.wasm
