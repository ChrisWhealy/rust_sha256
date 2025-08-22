# Rust+WASI Implementation of the SHA256 Algorithm

Having developed an implementation of the [SHA256 algorithm directly in WebAssembly Text](https://github.com/ChrisWhealy/wasm_sha256), I was pleased to get the compiled binary down to about 2.5Kb.

By way of comparison, I decided to implement the equivalent functionality in Rust, compile it to WebAssembly and then look at the size of the WASM binary.

As a result, I now have two versions of the Rust implementation that implement file I/O using different libraries:
* The standard Rust library
* The WASI library

The standard library version can be run using `cargo run` as you would with any other Rust program, but to run either version from WebAssembly, they must be compiled for the `wasm32-wasip1` target, then run using a WebAssembly host environment such as `wasmer` or `wasmtime`. 

# TL;DR

## Using the Standard Rust Library for File I/O

Using the standard Rust library to perform all the file I/O is straight forward and does not require very much coding.
The optimized WebAssembly binary is about 72Kb.

## Using the WASI Library for File I/O

Using the WASI library instead to perform all the file I/O requires that you write a specific Rust wrapper function for each WASI function you wish to call.
This requires much more effort, but the optimized WebAssembly binary is 42Kb.  

## Using Handcrafted WebAssembly That interacts Directly with WASI

Look at the coding in this repo <https://github.com/ChrisWhealy/wasm_sha256>

The optimized WebAssembly binary is 2.5Kb.

😎

# Local Execution

## Prerequsites

Before this program can be run from WebAssembly, you must first install the `wasm32-wasip1` compilation target.

```bash
$ rustup target add wasm32-wasip1
info: downloading component 'rust-std' for 'wasm32-wasip1'
info: installing component 'rust-std' for 'wasm32-wasip1'
 21.3 MiB /  21.3 MiB (100 %)  14.7 MiB/s in  1s
```

## Run Without WebAssembly

Simply use `cargo run` to build and run the `std` version of the binary:

```bash
$ cargo run --bin std --release ./src/bin/std.rs
    Finished `release` profile [optimized] target(s) in 0.04s
     Running `target/release/std ./src/bin/std.rs`
008d580e17bb8da5bf3458037ab9e39b2a48ee2688bf004abf4529bf1c35ea1c  ./src/bin/std.rs
```

## Run From WebAssembly Using Rust `std`

```bash
$ ./build.sh std
Build std -> ./target/wasm32-wasip1/release/std.opt.wasm
   Compiling wit-bindgen-rt v0.39.0
   Compiling bitflags v2.9.2
   Compiling wee_alloc v0.4.5
   Compiling cfg-if v0.1.10
   Compiling memory_units v0.4.0
   Compiling wasi v0.14.2+wasi-0.2.4
   Compiling sha256 v1.0.0 (/Users/chris/Developer/rust/sha256)
    Finished `release` profile [optimized] target(s) in 3.60s
```

The optimized WebAssembly module `./target/wasm32-wasip1/release/std.opt.wasm` is about 71Kb.

```bash
$ wasmer run ./target/wasm32-wasip1/release/std.opt.wasm --mapdir /::./src/bin -- std.rs
008d580e17bb8da5bf3458037ab9e39b2a48ee2688bf004abf4529bf1c35ea1c  std.rs
```

## Run From WebAssembly Using Rust `std`

```bash
$ ./build.sh wasi                                                                       
Build wasi -> ./target/wasm32-wasip1/release/wasi.opt.wasm
   Compiling sha256 v1.0.0 (/Users/chris/Developer/rust/sha256)
    Finished `release` profile [optimized] target(s) in 1.53s
```

The optimized WebAssembly module `./target/wasm32-wasip1/release/wasi.opt.wasm` is about 42Kb.

```bash
$ wasmer run ./target/wasm32-wasip1/release/std.opt.wasm --mapdir /::./src/bin -- std.rs
008d580e17bb8da5bf3458037ab9e39b2a48ee2688bf004abf4529bf1c35ea1c  std.rs
```

In spite of the fact that this binary is 1/3 smaller then , there still seems to be some unavoidable Rust/WASI baggage.

* The use of standard Rust functionality is convenient, but the cost of such convenience is the fact that every time you do "normal" things like allocate a `String` or use the `format!` macro, you implicitly bring into scope a large amount of extra coding that could (with some effort) be removed. 
* The `wasm32-wasip1` compilation target adds sections to the `.wasm` module for things like `__data_end`, `__heap_base`, and metadata that are not needed in the hand-crafted `.wat` implementation.
