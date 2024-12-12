BaseNC
======

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/basenc.svg)](https://crates.io/crates/basenc)
[![docs.rs](https://docs.rs/basenc/badge.svg)](https://docs.rs/basenc)
[![Build status](https://github.com/CasualX/basenc/workflows/CI/badge.svg)](https://github.com/CasualX/basenc/actions)

Pronounced "Base-En-See".

Encoding and decoding hex, base64 and base32 with support for `#[no_std]`.

Examples
--------

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

* `std` (default) - Enable support for the standard library. This enables convenience features to encode and decode to `String` and `Vec<u8>` buffers.

* `simd-off` - Disable SIMD acceleration. The SIMD codepaths are less tested and are more likely to contain bugs.

* `simd-runtime` - Enable runtime detection of SIMD support. This is enabled by default and will automatically use SIMD acceleration if available.

Build with `RUSTFLAGS="-C target-cpu=native"` (bash) / `set RUSTFLAGS=-C target-cpu=native` (cmd) to enable compiletime detection of SIMD capabilities.

Future work
-----------

Profile and optimize for performance.

Implement SIMD accelerated algorithms for encoding and decoding.

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
