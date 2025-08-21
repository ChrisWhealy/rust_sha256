#!/bin/sh
set -e

target=wasm32-wasip1
bin_name=sha256

RUSTFLAGS="-C link-arg=-s" cargo build --release --target $target
wasm-opt ./target/$target/release/${bin_name}.wasm --strip-debug --strip-dwarf --enable-simd --enable-multivalue --enable-bulk-memory -O4 -o ./target/$target/release/${bin_name}.opt.wasm
# wasm-strip ./target/$target/release/${bin_name}.opt.wasm
