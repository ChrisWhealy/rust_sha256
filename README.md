# Rust Implementation of the SHA256 Algorithm

Having developed an implementation of the [SHA256 algorithm directly in WebAssembly Text](https://github.com/ChrisWhealy/wasm_sha256), I was pleased to get the compiled binary down to just under 3Kb.

Therefore, I decided to implement the equivalent functionality in Rust, compile it to WebAssembly and see what the size difference is of the two binaries.

Both `.wasm` binaries have been run through `wasm-opt` using the same options, but still the size difference is just over a factor of 20.

## Build

1. `rustup add target wasip1`
2. `cargo build --target=wasm32-wasip1 --release`
3.  ```bash
    wasm-opt ./target/wasm32-wasip1/release/sha256.wasm \
    --enable-simd --enable-multivalue --enable-bulk-memory -O4 \
    -o ./target/wasm32-wasip1/release/sha256_opt.wasm
    ```
4. ```bash
   ll ./target/wasm32-wasip1/release
   total 448
   drwxr-xr-x  11 chris  staff    352 19 Aug 13:27 .
   drwxr-xr-x@  5 chris  staff    160 19 Aug 12:03 ..
   -rw-r--r--   1 chris  staff      0 19 Aug 12:03 .cargo-lock
   drwxr-xr-x   4 chris  staff    128 19 Aug 13:27 .fingerprint
   drwxr-xr-x   2 chris  staff     64 19 Aug 12:03 build
   drwxr-xr-x   6 chris  staff    192 19 Aug 13:27 deps
   drwxr-xr-x   2 chris  staff     64 19 Aug 12:03 examples
   drwxr-xr-x   2 chris  staff     64 19 Aug 12:03 incremental
   -rw-r--r--   1 chris  staff  66620 19 Aug 13:28 sha256_opt.wasm
   -rw-r--r--   1 chris  staff    124 19 Aug 12:03 sha256.d
   -rwxr-xr-x   1 chris  staff  91474 19 Aug 13:27 sha256.wasm
   ```

As you can see, the optimized `.wasm` binary is about 65Kb compared to the 2.9Kb of the hand-crafted WAT file...

## Running from a WebAssembly Runtime Environment

Once you have compiled the Rust coding to the `wasip1` target, then run the generated binary through `wasm-opt`, the WebAssembly module can be run from these WebAssembly runtime environments.

### Wasmer

```bash
$ wasmer run ./target/wasm32-wasip1/release/sha256_opt.wasm --mapdir /::./src main.rs   
8a8b9880912758007361707123da3cd0be40bf4bbd04c546a55fce0a6e1a405e  main.rs
```

### Wasmtime

```bash
$ wasmtime --dir ./src ./target/wasm32-wasip1/release/sha256_opt.wasm ./src/main.rs
8a8b9880912758007361707123da3cd0be40bf4bbd04c546a55fce0a6e1a405e  ./src/main.rs
```

### Wazero

```bash
$ wazero run -mount=.:. ./target/wasm32-wasip1/release/sha256_opt.wasm ./src/main.rs 
8a8b9880912758007361707123da3cd0be40bf4bbd04c546a55fce0a6e1a405e  ./src/main.rs
```
