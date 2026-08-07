BaseNC
======

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/basenc.svg)](https://crates.io/crates/basenc)
[![docs.rs](https://docs.rs/basenc/badge.svg)](https://docs.rs/basenc)
[![Build status](https://github.com/CasualX/basenc/actions/workflows/gate.yml/badge.svg)](https://github.com/CasualX/basenc/actions/workflows/gate.yml)

Pronounced **"Base-En-See"**.

Encoding and decoding **hex**, **base64** and **base32** with support for `#[no_std]`.

📦 Installation
---------------

```toml
[dependencies]
basenc = "1.0.0-alpha.1"
```

BaseNC requires Rust 1.95 or newer.

🌟 Why BaseNC?
--------------

* Optimized performance – SIMD-accelerated algorithms for blazing-fast encoding/decoding on x86, aarch64 and wasm32.
* Zero dependencies – Lightweight and reliable, no extra baggage.
* Simple, ergonomic API – Encode/decode in just a few lines of code.
* #[no_std] support – Works seamlessly in embedded and constrained environments.

🚀 Examples
-----------

Encoding:

```rust
let encoded = basenc::Base64Std.encode(b"hello world");
assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
```

Decoding:

```rust
let decoded = basenc::Base64Std.decode("aGVsbG8gd29ybGQ=").unwrap();
assert_eq!(decoded, b"hello world");
```

The standard Base64 and Base32 encodings emit padding by default. Select another policy explicitly when needed:

```rust
use basenc::{Base64Std, Padding};

let encoded = Base64Std.pad(Padding::Forbidden).encode(b"hello world");
assert_eq!(encoded, "aGVsbG8gd29ybGQ");
```

### Features

* `std` (default) - Enable support for the standard library, including convenient encoding/decoding to `String` and `Vec<u8>`.

* `simd-runtime` (default) - Enable runtime detection of SIMD support. This feature requires `std`, and will automatically use SIMD acceleration when available.

* `simd-off` - Disable SIMD acceleration. (The SIMD paths are less tested and may contain bugs.)

Tip: Build with `RUSTFLAGS="-C target-cpu=native"` (bash) or `set RUSTFLAGS=-C target-cpu=native` (cmd) to **enable compile-time detection**.

For WebAssembly SIMD, build with `RUSTFLAGS="-C target-feature=+simd128" cargo build --target wasm32-unknown-unknown`.
Wasm SIMD is selected at compile time; engines which do not support SIMD128 reject the module during validation,
so applications that need a scalar fallback should ship a second build and select between them with `WebAssembly.validate` or feature detection in their loader.

The [`wasm-tests`](wasm-tests/) standalone crate builds scalar and SIMD128 modules for correctness comparison and optional Node benchmarks.

📜 License
----------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
