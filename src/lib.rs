/*!
BaseNC
======

Encoding and decoding of Base-N encodings, `#[no_std]` compatible.

Examples
--------

Encoding:

```
assert_eq!(
	basenc::Base64Std.encode(b"hello world"),
	"aGVsbG8gd29ybGQ",
);
```

Decoding:

```
assert_eq!(
	basenc::Base64Std.decode("aGVsbG8gd29ybGQ=").unwrap(),
	b"hello world",
);
```

Encoding trait
--------------

The hero of the show is [`Encoding`], defining the entry point for encoding and decoding for an encoding.

Buffers
-------

Existing buffers can be reused with the [`encode_into`](Encoding::encode_into) and [`decode_into`](Encoding::decode_into) methods.

Buffers are types implementing the [`EncodeBuf`] and [`DecodeBuf`] traits.

Under `#[no_std]` they are only implemented by `&mut [u8]` acting as a fixed size buffer.

Otherwise [`EncodeBuf`] is implemented by `String` for convenience and `&mut String` and `&mut Vec<u8>` for efficient buffer reuse.
[`DecodeBuf`] is implemented by `Vec<u8>` for convenience and `&mut Vec<u8>` for efficient buffer reuse.

*/

#![no_std]

#[cfg(any(test, feature = "std"))]
#[macro_use]
extern crate std;

macro_rules! impl_encoding {
	(
		$name:path
		$(,
		encode: [$($encode_example:literal,)*],
		decode: [$($decode_example:literal,)*],
		encode_into: [$($encode_into_example:literal,)*],
		decode_into: [$($decode_into_example:literal,)*],
		)?
	) => {
		impl $name {
			#[cfg(feature = "std")]
			/// Encodes the input bytes.
			$(
				///
				/// # Examples
				$(#[doc = $encode_example])*
			)?
			#[inline]
			pub fn encode(&self, bytes: &[u8]) -> std::string::String {
				crate::Encoding::encode_into(self, bytes, std::string::String::new())
			}

			#[cfg(feature = "std")]
			/// Decodes the input string.
			$(
				///
				/// # Examples
				$(#[doc = $decode_example])*
			)?
			#[inline]
			pub fn decode(&self, string: &str) -> Result<std::vec::Vec<u8>, crate::Error> {
				crate::Encoding::decode_into(self, string, std::vec::Vec::new())
			}

			/// Encodes into a buffer.
			$(
				///
				/// # Examples
				$(#[doc = $encode_into_example])*
			)?
			#[inline]
			pub fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
				crate::Encoding::encode_into(self, bytes, buffer)
			}

			/// Decodes into a buffer.
			$(
				///
				/// # Examples
				$(#[doc = $decode_into_example])*
			)?
			#[inline]
			pub fn decode_into<B: crate::DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, crate::Error> {
				crate::Encoding::decode_into(self, string, buffer)
			}
		}
	};
}

mod buf;
pub use self::buf::{EncodeBuf, DecodeBuf};

mod hex;
pub use self::hex::{LowerHex, UpperHex};

mod base64;
pub use self::base64::*;

/// Decoding error.
///
/// Note that encoding can never fail.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
	/// Not a valid character in the alphabet.
	InvalidChar,
	/// Input has incorrect length or is not padded to the required length.
	BadLength,
	/// Input is not canonical.
	///
	/// Unused padding MUST consist of zero bits.
	Denormal,
}

/// Padding policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Padding {
	/// No padding.
	None,
	/// Optional padding.
	///
	/// Padding accepted while decoding, not added while encoding.
	Optional,
	/// Strict padding.
	Strict,
}

//----------------------------------------------------------------

/// Data encoding.
pub trait Encoding {
	/// Encodes into an encoding buffer.
	fn encode_into<B: EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output;

	/// Decodes into a decoding buffer.
	fn decode_into<B: DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, Error>;
}

#[cold]
fn panic_overflow() -> ! {
	panic!("overflow")
}
