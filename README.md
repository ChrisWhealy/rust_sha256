# Rust Implementation of the SHA256 Algorithm

Having developed an implementation of the [SHA256 algorithm directly in WebAssembly Text](https://github.com/ChrisWhealy/wasm_sha256), I was pleased to get the compiled binary down to about 2.5Kb.

By way of comparison, I decided to implement the equivalent functionality in Rust using only the WASI interface, compile it to WebAssembly and look at the size of the WASM binary.

Interacting only with the WASI libraru means this program can no longer be compiled for a default target such as `x86_64-apple-darwin`: it can only ever be compiled for the `wasm32-wasip1` target.

## Build

1. `rustup add target wasip1`
2. [`./build-opt.sh`](https://github.com/ChrisWhealy/rust_sha256/blob/master/build-opt.sh)
4. ```bash
   ll target/wasm32-wasip1/release/                                                  
   total 416
   drwxr-xr-x  11 chris  staff    352 21 Aug 17:50 .
   drwxr-xr-x@  5 chris  staff    160 21 Aug 09:50 ..
   -rw-r--r--   1 chris  staff      0 20 Aug 16:32 .cargo-lock
   drwxr-xr-x  11 chris  staff    352 20 Aug 16:32 .fingerprint
   drwxr-xr-x   4 chris  staff    128 20 Aug 16:32 build
   drwxr-xr-x  22 chris  staff    704 21 Aug 17:50 deps
   drwxr-xr-x   2 chris  staff     64 20 Aug 16:32 examples
   drwxr-xr-x   2 chris  staff     64 20 Aug 16:32 incremental
   -rw-r--r--   1 chris  staff    224 21 Aug 17:50 sha256.d
   -rw-r--r--   1 chris  staff  69178 21 Aug 17:50 sha256.opt.wasm
   -rwxr-xr-x   1 chris  staff  74041 21 Aug 17:50 sha256.wasm
   ```

Now compare this with the binary size from the [handcrafted WebAssembly Text version](https://github.com/ChrisWhealy/wasm_sha256/tree/main/bin).

```bash
$ ll ./bin 
total 16
drwxr-xr-x   4 chris  staff   128 21 Aug 11:48 .
drwxr-xr-x  20 chris  staff   640 21 Aug 11:00 ..
-rw-r--r--   1 chris  staff  2598 21 Aug 11:48 sha256.opt.wasm
-rw-r--r--   1 chris  staff  3613 21 Aug 11:48 sha256.wasm
```

In both cases, `sha256.opt.wasm` has been created by running `sha256.wasm` through `wasm-opt`.
But even after replacing all usages `std::io::{READ, Write} ` with the corresponding WASI calls (`path_open`, `fd_read` etc), the `.wasm` file generated from Rust is still about 25 times larger.

To reduce this any further, the Rust code needs to switch to `no-std`.
That said, there still seems to be some unavoidable Rust/WASI baggage:

Having `wasm32-wasip` as the compilation target means WASM section are generated for things like `__data_end`, `__heap_base`, and metadata that are unneeded in a hand-crafted `.wat` implementation.

## Running from a WebAssembly Runtime Environment

Run `./build-opt.sh`, then run the generated binary from the WebAssembly runtime of your choice.
For example:

### Wasmer

```bash
$ wasmer run ./target/wasm32-wasip1/release/sha256.opt.wasm --mapdir /::./src main.rs   
5e31f29a050fd34f09081bca07edad2b0a4936a2b3bee0b769a430fcd507322b  main.rs
```

### Wasmtime

```bash
$ wasmtime --dir ./src ./target/wasm32-wasip1/release/sha256.opt.wasm ./src/main.rs
5e31f29a050fd34f09081bca07edad2b0a4936a2b3bee0b769a430fcd507322b  ./src/main.rs
```

### Wazero

```bash
$ wazero run -mount=.:. ./target/wasm32-wasip1/release/sha256.opt.wasm ./src/main.rs 
5e31f29a050fd34f09081bca07edad2b0a4936a2b3bee0b769a430fcd507322b  ./src/main.rs
```
