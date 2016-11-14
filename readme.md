BaseNC
======

Pronounced "Base-En-See".

Encoding and decoding hex, base64 and base32 with support for `#[no_std]`.

Usage
-----

The documentation can be found on [docs.rs](https://docs.rs/basenc/).

This library can be found on [crates.io](https://crates.io/crates/basenc).

In your Cargo.toml put

```
[dependencies]
basenc = "0.1"
```

### Features

Features available with Cargo.toml

```
[dependencies.basenc]
version = "0.1"
default-features = false
features = ["std", "lut"]
```

* `std` - Enable support for the standard library. This enables convenience features to encode and decode to `String` and `Vec<u8>` buffers.

* `lut` - Use lookup tables instead of chained comparisons for the translation.

* `unstable` - Expose the unstable inner details of this library. Build docs with this feature to get its documentation.

The default features are [`std`, `lut`]. To enable `#[no_std]` requires disabling default features.

Future work
-----------

Implement base32 encoding.

Implement better support for esotheric base64 encoding variants.

Profile and optimize for performance.

License
-------

MIT, see license.txt
