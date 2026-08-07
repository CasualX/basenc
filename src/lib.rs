/*!
BaseNC
======

Pronounced **"Base-En-See"**.

Encoding and decoding **hex**, **base64** and **base32** with support for #\[no_std\].

Examples
--------

Encoding:

```
let encoded = basenc::Base64Std.encode(b"hello world");
assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
```

Decoding:

```
let decoded = basenc::Base64Std.decode("aGVsbG8gd29ybGQ=").unwrap();
assert_eq!(decoded, b"hello world");
```

Encoding
--------

The hero of the show is [`Encoding`], defining the entry point for encoding and decoding for an encoding.

Buffers
-------

Buffers are types implementing the [`EncodeBuf`] and [`DecodeBuf`] traits.

Existing buffers can be reused with the [`encode_bytes_into`](Encoding::encode_bytes_into) and [`decode_bytes_into`](Encoding::decode_bytes_into) methods.

*/

#![cfg_attr(not(any(test, feature = "std")), no_std)]

#![allow(unsafe_op_in_unsafe_fn)] // All unsafe fn use unsafe code inside

#[allow(unused_imports)]
use core::{fmt, mem, ptr, slice, str};

#[macro_use]
mod encoding;

#[macro_use]
mod arch;

mod ratio;
pub use self::ratio::Ratio;

mod pad;
pub use self::pad::*;
pub use Padding::Forbidden as NoPad;

mod buf;
pub use self::buf::*;

mod hex;
pub use self::hex::*;

mod base64;
pub use self::base64::*;

mod base64std;
pub use self::base64std::*;

mod base32;
pub use self::base32::*;

#[cfg(doc)]
pub mod incremental;

//----------------------------------------------------------------

/// The kind of decoding error.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
	/// Not a valid character in the alphabet.
	InvalidCharacter,
	/// Input has incorrect length or is not padded to the required length.
	IncorrectLength,
	/// Input is not canonical.
	///
	/// Unused padding MUST consist of zero bits.
	NonCanonical,
}

impl fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match self {
			ErrorKind::InvalidCharacter => "invalid character",
			ErrorKind::IncorrectLength => "incorrect length",
			ErrorKind::NonCanonical => "non-canonical input",
		})
	}
}

/// Decoding error.
///
/// Note that encoding can never fail.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Error {
	/// The kind of error that occurred.
	pub kind: ErrorKind,
	/// Zero-based byte offset where the error was encountered.
	///
	/// An offset equal to the input length indicates an unexpected end of input.
	pub offset: usize,
}

impl Error {
	/// Creates a decoding error.
	#[inline]
	pub const fn new(kind: ErrorKind, offset: usize) -> Self {
		Self { kind, offset }
	}

	#[inline]
	pub(crate) const fn shifted(self, offset: usize) -> Self {
		Self { offset: self.offset + offset, ..self }
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} at offset {}", self.kind, self.offset)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

//----------------------------------------------------------------

/// Display wrapper for encoding.
#[derive(Clone, Debug)]
pub struct Display<'a, E> {
	encoding: &'a E,
	bytes: &'a [u8],
}

impl<'a, E: Encoding> Display<'a, E> {
	/// Wraps the encoding and bytes for display.
	#[inline]
	pub fn new(encoding: &'a E, bytes: &'a [u8]) -> Self {
		Self { encoding, bytes }
	}
}

impl<'a, E: Encoding> fmt::Display for Display<'a, E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut stack_buf = mem::MaybeUninit::<[u8; 512]>::uninit();
		let chunk_size = E::RATIO.encoding_chunk_size(mem::size_of_val(&stack_buf));

		for chunk in self.bytes.chunks(chunk_size) {
			let string = self.encoding.encode_bytes_into(chunk, &mut stack_buf);
			f.write_str(string)?;
		}

		Ok(())
	}
}

//----------------------------------------------------------------

mod sealed {
	pub trait Sealed {}
}
use sealed::Sealed;

/// Data encoding.
pub trait Encoding: Sealed {
	/// Encoding ratio of decoded to encoded bytes.
	const RATIO: Ratio;

	/// Encodes into an encoding buffer.
	fn encode_bytes_into<B: EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output;

	/// Decodes into a decoding buffer.
	fn decode_bytes_into<B: DecodeBuf>(&self, string: &[u8], buffer: B) -> Result<B::Output, Error>;
}
