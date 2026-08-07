# WebAssembly SIMD tests

This standalone crate builds the same correctness harness with scalar and SIMD128 implementations.
The Node runner executes both modules, checks error handling and buffer boundaries, and compares deterministic output fingerprints.

From this directory:

```console
CARGO_TARGET_DIR=target/scalar cargo build --release --target wasm32-unknown-unknown --features scalar
CARGO_TARGET_DIR=target/simd RUSTFLAGS="-C target-feature=+simd128" cargo build --release --target wasm32-unknown-unknown
node run.mjs
```

Pass `--bench` to print a small comparative throughput benchmark:

```console
node run.mjs --bench
```
