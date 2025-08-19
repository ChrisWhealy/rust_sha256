#!/bin/sh
set -e
RUSTFLAGS="-C link-arg=-s" cargo build --release --target wasm32-wasip1
wasm-opt ./target/wasm32-wasip1/release/sha256.wasm --strip-debug --strip-dwarf --enable-simd --enable-multivalue --enable-bulk-memory -Oz -o ./target/wasm32-wasip1/release/sha256_opt.wasm
wasm-strip ./target/wasm32-wasip1/release/sha256_opt.wasm
