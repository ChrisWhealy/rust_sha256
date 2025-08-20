# Rust Implementation of the SHA256 Algorithm

Having developed an implementation of the [SHA256 algorithm directly in WebAssembly Text](https://github.com/ChrisWhealy/wasm_sha256), I was pleased to get the compiled binary down to just under 3Kb.

Therefore, I decided to implement the equivalent functionality in Rust, compile it to WebAssembly and see what the size difference is of the two binaries.

## Build

1. `rustup add target wasip1`
2. `./build-opt.sh`
4. ```bash
   ll target/wasm32-wasip1/release/               
   total 256
   drwxr-xr-x  11 chris  staff    352 20 Aug 16:33 .
   drwxr-xr-x@  4 chris  staff    128 20 Aug 16:32 ..
   -rw-r--r--   1 chris  staff      0 20 Aug 16:32 .cargo-lock
   drwxr-xr-x  11 chris  staff    352 20 Aug 16:32 .fingerprint
   drwxr-xr-x   4 chris  staff    128 20 Aug 16:32 build
   drwxr-xr-x  22 chris  staff    704 20 Aug 16:32 deps
   drwxr-xr-x   2 chris  staff     64 20 Aug 16:32 examples
   drwxr-xr-x   2 chris  staff     64 20 Aug 16:32 incremental
   -rw-r--r--   1 chris  staff    124 20 Aug 16:32 sha256.d
   -rw-r--r--   1 chris  staff  57542 20 Aug 16:33 sha256.opt.wasm
   -rwxr-xr-x   1 chris  staff  65015 20 Aug 16:32 sha256.wasm
   ```

Now compare this with the binary size from the [handcrafted WebAssembly Text version](https://github.com/ChrisWhealy/wasm_sha256/tree/main/bin).

```bash
$ ll ./bin 
total 40
drwxr-xr-x   6 chris  staff   192 20 Aug 14:09 .
drwxr-xr-x  20 chris  staff   640 20 Aug 15:16 ..
-rw-r--r--   1 chris  staff  3058 20 Aug 14:09 sha256.debug.opt.wasm
-rw-r--r--   1 chris  staff  6633 20 Aug 14:09 sha256.debug.wasm
-rw-r--r--   1 chris  staff  2535 20 Aug 14:09 sha256.opt.wasm
-rw-r--r--   1 chris  staff  2817 20 Aug 14:09 sha256.wasm
```

In both cases, `sha256.opt.wasm` has been created by running `sha256.wasm` through `wasm-opt`.
But even after applying some (not all, admittedly) optimisations, the `.wasm` file generated from Rust is still about 20 times larger.

To reduce this any further, the Rust code needs to abandon the use of `std` and interact only with the `WASI` library.

The first step has been taken here and the command line arguments are fetched by calling WASI's `args_sizes_get`.
However, this now means that the program can no longer be compiled for a default target such as `x86_64-apple-darwin`: it can only ever be compiled for the `wasm32-wasip1` target.

However, to reduce this any further, significant effort would be needed to create a `no-std` implementation that performs all system interaction directly through `WASI`.

There seems to be some unavoidable Rust/WASI baggage:

1. **Rust runtime and `std` baggage.**<br>
   Even with `wee_alloc`, `panic = "abort"`, and `-Oz`, Rust still includes the WASI syscall wrappers, global allocator scaffolding, and memory initialization routines.

   This will consume a few tens of KB.

1. **WASM target overhead**<br>
   `wasm32-wasip1` emits sections for things like `__data_end`, `__heap_base`, and metadata that a hand-crafted `.wat` doesn’t need at all.

## Running from a WebAssembly Runtime Environment

Once you have compiled the Rust coding to the `wasip1` target, then run the generated binary through `wasm-opt`, the WebAssembly module can be run from these WebAssembly runtime environments.

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
