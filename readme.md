BaseNC
======

[![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/urandom.svg)](https://crates.io/crates/urandom)
[![docs.rs](https://docs.rs/urandom/badge.svg)](https://docs.rs/urandom)

Pronounced "Base-En-See".

Encoding and decoding hex, base64 and base32 with support for `#[no_std]`.

Examples
--------

Encoding:

```rust
assert_eq!(
	basenc::Base64Std.encode(b"hello world"),
	"aGVsbG8gd29ybGQ",
);
```

Decoding:

```rust
assert_eq!(
	basenc::Base64Std.decode("aGVsbG8gd29ybGQ=").unwrap(),
	b"hello world",
);
```

### Features

* `std` (default) - Enable support for the standard library. This enables convenience features to encode and decode to `String` and `Vec<u8>` buffers.

Future work
-----------

Implement base32 encoding.

Implement better support for esotheric base64 encoding variants.

Profile and optimize for performance.

License
-------

MIT, see license.txt
