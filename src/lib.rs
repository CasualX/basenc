/*!
BaseNC
======

Encoding and decoding of Base-N encodings, `#[no_std]` compatible.

Examples
--------

Encoding:

```
let encoded = basenc::Base64Std.encode(b"hello world", basenc::Padding::Optional);
assert_eq!(encoded, "aGVsbG8gd29ybGQ");
```

Decoding:

```
let decoded = basenc::Base64Std.decode("aGVsbG8gd29ybGQ=", basenc::Padding::Optional).unwrap();
assert_eq!(decoded, b"hello world");
```

Encoding
--------

The hero of the show is [`Encoding`], defining the entry point for encoding and decoding for an encoding.

Buffers
-------

Buffers are types implementing the [`EncodeBuf`] and [`DecodeBuf`] traits.

Existing buffers can be reused with the [`encode_into`](Encoding::encode_into) and [`decode_into`](Encoding::decode_into) methods.

*/

#![no_std]

#[allow(unused_imports)]
use core::{fmt, mem, ptr, slice, str};

#[cfg(any(test, feature = "std"))]
#[macro_use]
extern crate std;

#[macro_use]
mod encoding;

#[macro_use]
mod arch;

mod ratio;
pub use self::ratio::Ratio;

mod buf;
pub use self::buf::*;

mod hex;
pub use self::hex::*;

mod base64;
pub use self::base64::*;

mod base32;
pub use self::base32::*;

pub mod incremental;

//----------------------------------------------------------------

/// Decoding error.
///
/// Note that encoding can never fail.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
	/// Not a valid character in the alphabet.
	InvalidCharacter,
	/// Input has incorrect length or is not padded to the required length.
	IncorrectLength,
	/// Input is not canonical.
	///
	/// Unused padding MUST consist of zero bits.
	NonCanonical,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
			Error::InvalidCharacter => "invalid character",
			Error::IncorrectLength => "incorrect length",
			Error::NonCanonical => "non-canonical input",
		})
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

//----------------------------------------------------------------

/// Padding policy.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Padding {
	/// No padding.
	None,
	/// Optional padding.
	///
	/// Padding accepted while decoding, not added while encoding.
	#[default]
	Optional,
	/// Strict padding.
	Strict,
}

pub use self::Padding::None as NoPad;

//----------------------------------------------------------------

/// Display wrapper for encoding.
#[derive(Clone, Debug)]
pub struct Display<'a, E> {
	encoding: &'a E,
	bytes: &'a [u8],
	pad: Padding,
}

impl<'a, E: Encoding> Display<'a, E> {
	/// Wraps the encoding and bytes for display.
	#[inline]
	pub fn new(encoding: &'a E, bytes: &'a [u8], pad: Padding) -> Self {
		Self { encoding, bytes, pad }
	}
}

impl<'a, E: Encoding> fmt::Display for Display<'a, E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut stack_buf = mem::MaybeUninit::<[u8; 512]>::uninit();
		let chunk_size = E::RATIO.encoding_chunk_size(mem::size_of_val(&stack_buf));

		for chunk in self.bytes.chunks(chunk_size) {
			let string = self.encoding.encode_into(chunk, self.pad, &mut stack_buf);
			f.write_str(string)?;
		}

		Ok(())
	}
}

//----------------------------------------------------------------

/// Data encoding.
pub trait Encoding {
	/// Encoding ratio of decoded to encoded bytes.
	const RATIO: Ratio;

	/// Encodes into an encoding buffer.
	fn encode_into<B: EncodeBuf>(&self, bytes: &[u8], pad: Padding, buffer: B) -> B::Output;

	/// Decodes into a decoding buffer.
	fn decode_into<B: DecodeBuf>(&self, string: &[u8], pad: Padding, buffer: B) -> Result<B::Output, Error>;
}
