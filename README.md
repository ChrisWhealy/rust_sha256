# Rust Implementation of the SHA256 Algorithm

Having developed an implementation of the [SHA256 algorithm directly in WebAssembly Text](https://github.com/ChrisWhealy/wasm_sha256), I was pleased to get the compiled binary down to just under 3Kb.

Therefore, I decided to implement the equivalent functionality in Rust, compile it to WebAssembly and see what the size difference is of the two binaries.

Both `.wasm` binaries have been run through `wasm-opt` using the same options, but even with some (not all, admittedly) optimisations, the `.wasm` file coming from Rust was still just over 18 times larger.

To reduce this any further, the Rust code needs to abandon the use of `std` and interact directly with the `WASI` library.

The first step has been taken here and the command line arguments are fetched by calling WASI's `args_sizes_get`.
However, this now means that the program can no longer be compiled for a default target such as `x86_64-apple-darwin`.
It is now completely dependent on WASI, so it can only ever be compiled for the `wasm32-wasip1` target.  

## Build

1. `rustup add target wasip1`
2. `./build-opt.sh`
4. ```bash
   ll ./target/wasm32-wasip1/release                                                  
   total 248
   drwxr-xr-x  11 chris  staff    352 19 Aug 17:24 .
   drwxr-xr-x@  5 chris  staff    160 19 Aug 12:03 ..
   -rw-r--r--   1 chris  staff      0 19 Aug 12:03 .cargo-lock
   drwxr-xr-x  27 chris  staff    864 19 Aug 17:24 .fingerprint
   drwxr-xr-x   6 chris  staff    192 19 Aug 17:24 build
   drwxr-xr-x  59 chris  staff   1888 19 Aug 17:24 deps
   drwxr-xr-x   2 chris  staff     64 19 Aug 12:03 examples
   drwxr-xr-x   2 chris  staff     64 19 Aug 12:03 incremental
   -rw-r--r--   1 chris  staff  57118 19 Aug 17:24 sha256_opt.wasm
   -rw-r--r--   1 chris  staff    124 19 Aug 12:03 sha256.d
   -rwxr-xr-x   1 chris  staff  65015 19 Aug 17:24 sha256.wasm
   ```

As you can see, the optimized `.wasm` binary is about 55Kb compared to the 2.9Kb of the hand-crafted WAT file...

However, to reduce this any further, significant effort would be needed to create a `no-std` implementation that interacts directly with WASI

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
