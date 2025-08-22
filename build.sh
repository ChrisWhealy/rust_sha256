#!/bin/sh
set -e

target=wasm32-wasip1

if [ $# -ne 1 ]; then
  echo "Error: Expected exactly 1 argument, got $#"
  echo "Usage: $0 std | wasi"
  exit 1
fi

bin_name=$1

case "$bin_name" in
  "wasi"|"std")
    echo "Build $bin_name -> ./target/${target}/release/${bin_name}.opt.wasm"
    RUSTFLAGS="-C link-arg=-s" cargo build --bin $bin_name --release --target $target
    wasm-opt ./target/$target/release/${bin_name}.wasm --strip-debug --strip-dwarf --enable-bulk-memory -O4 -o ./target/$target/release/${bin_name}.opt.wasm
    ;;
  *)
    echo "Invalid build option '$bin_name'"
    echo "Expected: std or wasi"
    ;;
esac
