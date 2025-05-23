BaseNC
======

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/basenc.svg)](https://crates.io/crates/basenc)
[![docs.rs](https://docs.rs/basenc/badge.svg)](https://docs.rs/basenc)
[![Build status](https://github.com/CasualX/basenc/workflows/CI/badge.svg)](https://github.com/CasualX/basenc/actions)

Pronounced **"Base-En-See"**.

Encoding and decoding **hex**, **base64** and **base32** with support for #[no_std].

🌟 Why BaseNC?
--------------

* ⚡ Optimized performance – SIMD-accelerated algorithms for blazing-fast encoding/decoding.
* 📦 Zero dependencies – Lightweight and reliable, no extra baggage.
* 🦀 Simple, ergonomic API – Encode/decode in just a few lines of code.
* 🔧 #[no_std] support – Works seamlessly in embedded and constrained environments.

🚀 Examples
-----------

Encoding:

```rust
let encoded = basenc::Base64Std.encode(b"hello world");
assert_eq!(encoded, "aGVsbG8gd29ybGQ");
```

Decoding:

```rust
let decoded = basenc::Base64Std.decode("aGVsbG8gd29ybGQ=").unwrap();
assert_eq!(decoded, b"hello world");
```

### Features

* `std` (default) - Enable support for the standard library, including convenient encoding/decoding to `String` and `Vec<u8>`.

* `simd-off` - Disable SIMD acceleration. (The SIMD paths are less tested and may contain bugs.)

* `simd-runtime` - Enable runtime detection of SIMD support. This is **on by default**, and will automatically use SIMD acceleration when available.

Tip: Build with `RUSTFLAGS="-C target-cpu=native"` (bash) or `set RUSTFLAGS=-C target-cpu=native` (cmd) to **enable compiletime detection**.

📜 License
----------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
