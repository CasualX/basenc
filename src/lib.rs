/*!
BaseNC
======

Encoding and decoding of Base-N encodings, `#[no_std]` compatible.

Encoding trait
--------------

The hero of the show is [`Encoding`](trait.Encoding.html), defining the entry point for encoding and decoding for an encoding.

The trait is implemented by unit structs, eg. `Base64Std`, allowing type inference to help out.

The hero has two sidekicks: [`Encoder`](trait.Encoder.html) and [`Decoder`](trait.Decoder.html) providing access to the encoding's iterator adapters.

Buffer to buffer
----------------

When you have an input buffer, `&[u8]` or `&str`, and want to encode or decode respectively to a new buffer
can be done conveniently using the [`encode`](fn.encode.html) and [`decode`](fn.decode.html) free functions.

They are ensured to be implemented efficiently and avoid code bloat.

### Buffers

A side note about buffers, they are types implementing the [`EncodeBuf`](trait.EncodeBuf.html) and [`DecodeBuf`](trait.DecodeBuf.html) traits.

Under `#[no_std]` they are only implemented by `&mut [u8]` acting as a fixed size buffer.

Otherwise `EncodeBuf` is implemented by `String` for convenience and `&mut String` and `&mut Vec<u8>` for efficient buffer reuse.
`DecodeBuf` is implemented by `Vec<u8>` for convenience and `&mut Vec<u8>` for efficient buffer reuse.

Iterator adapters
-----------------

For maximum flexibility the encoding and decoding can be pipelined as an iterator adapter.

The trait [`Encode`](trait.Encode.html) adapts an iterator over bytes given an encoding into an iterator over chars of the encoded input.

The trait [`Decode`](trait.Decode.html) adapts an iterator over chars given an encoding into an iterator over the resulting bytes of the decoded input.

*/

#![no_std]

#[cfg(any(test, feature = "std"))]
#[macro_use]
extern crate std;

#[cfg(feature = "unstable")]
pub mod details;
#[cfg(not(feature = "unstable"))]
mod details;

mod buf;
pub use buf::{EncodeBuf, DecodeBuf};

//----------------------------------------------------------------

/// Decoding error.
///
/// Note that encoding can never fail.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
	/// Not a valid character in the alphabet.
	InvalidChar(char),
	/// Input has incorrect length or is not padded to the required length.
	BadLength,
	/// Input is not canonical.
	///
	/// Unused padding MUST consist of zero bits.
	Denormal,
}

use ::core::fmt;
impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::InvalidChar(chr) => write!(f, "invalid char: {}", chr),
			Error::BadLength => write!(f, "bad length"),
			Error::Denormal => write!(f, "denormal"),
		}
	}
}
#[cfg(any(test, feature = "std"))]
impl ::std::error::Error for Error {
	fn description(&self) -> &str {
		match *self {
			Error::InvalidChar(_) => "invalid char",
			Error::BadLength => "bad length",
			Error::Denormal => "denormal",
		}
	}
}

//----------------------------------------------------------------

/// Data encoding.
///
/// Use the free-standing functions to avoid having to drag in this trait.
pub trait Encoding {
	/// Returns the encoding's alphabet.
	///
	/// # Examples
	///
	/// ```
	/// use basenc::{Encoding};
	///
	/// assert_eq!(
	/// 	basenc::LowerHex.alphabet(),
	/// 	"0123456789abcdef"
	/// );
	/// ```
	fn alphabet(self) -> &'static str;

	/// Directly encode into an encode buffer.
	///
	/// Use [`encode`](fn.encode.html) for convenience.
	fn encode<B: EncodeBuf>(self, bytes: &[u8], buffer: B) -> B::Output;

	/// Directly decode into a decode buffer.
	///
	/// Use [`decode`](fn.decode.html) for convenience.
	fn decode<B: DecodeBuf>(self, string: &str, buffer: B) -> Result<B::Output, Error>;
}

/// Directly encode into an encode buffer.
///
/// Convenient as it doesn't require the [`Encoding`](trait.Encoding.html) trait to be imported.
///
/// Note that encoding can never fail.
///
/// # Examples
///
/// ```
/// assert_eq!(
/// 	basenc::encode(b"hello world", basenc::Base64Std, String::new()),
/// 	"aGVsbG8gd29ybGQ="
/// );
/// ```
///
/// Convenience, appends to a by-value buffer and returns that buffer.
///
/// ```
/// let mut str_buf = String::from("output: ");
/// assert_eq!(
/// 	basenc::encode(b"BuFfEr ReUsE!", basenc::Base64Url, &mut str_buf),
/// 	"QnVGZkVyIFJlVXNFIQ"
/// );
/// assert_eq!(str_buf, "output: QnVGZkVyIFJlVXNFIQ");
/// ```
///
/// Appends to an existing buffer, returns a reference to the encoded input.
///
/// ```
/// let mut stack_buf = [0u8; 16];
/// assert_eq!(
/// 	basenc::encode(b"\x00\x80\xFF\xDC", basenc::LowerHex, &mut stack_buf[..]),
/// 	"0080ffdc"
/// );
/// ```
///
/// Uses fixed-size arrays on the stack as a buffer, available with `#[no_std]`.
///
/// Panics if the buffer is too small to fit the output.
pub fn encode<C: Encoding, B: EncodeBuf>(bytes: &[u8], encoding: C, buffer: B) -> B::Output {
	encoding.encode(bytes, buffer)
}
/// Directly decode into a decode buffer.
///
/// Convenient as it doesn't require the [`Encoding`](trait.Encoding.html) trait to be imported.
///
/// Decoding may fail and produce an [`Error`](enum.Error.html) instead.
///
/// # Examples
///
/// ```
/// assert_eq!(
/// 	basenc::decode("aGVsbG8gd29ybGQ=", basenc::Base64Std, Vec::new()),
/// 	Ok(b"hello world"[..].to_vec())
/// );
/// // Note that the buffer is swallowed on error
/// assert_eq!(
/// 	basenc::decode("&'nv@l!d", basenc::Base64Std, Vec::new()),
/// 	Err(basenc::Error::InvalidChar('&'))
/// );
/// ```
///
/// Convenience, appends to a by-value buffer and returns that buffer.
///
/// ```
/// let mut byte_buf = vec![0x11, 0x22, 0x33];
/// assert_eq!(
/// 	basenc::decode("QnVGZkVyIFJlVXNFIQ", basenc::Base64Url, &mut byte_buf),
/// 	Ok(&b"BuFfEr ReUsE!"[..])
/// );
/// assert_eq!(byte_buf, b"\x11\x22\x33BuFfEr ReUsE!");
/// ```
///
/// Appends to an existing buffer, returns a reference to the decoded input.
///
/// ```
/// let mut stack_buf = [0u8; 16];
/// assert_eq!(
/// 	basenc::decode("0080FFDC", basenc::UpperHex, &mut stack_buf[..]),
/// 	Ok(&b"\x00\x80\xFF\xDC"[..])
/// );
/// ```
///
/// Uses fixed-size arrays on the stack as a buffer, available with `#[no_std]`.
///
/// Panics if the buffer is too small to fit the output.
pub fn decode<C: Encoding, B: DecodeBuf>(string: &str, encoding: C, buffer: B) -> Result<B::Output, Error> {
	encoding.decode(string, buffer)
}

//----------------------------------------------------------------

/// Create an encoder adapter for an encoding given an `Iterator<Item = u8>`.
///
/// Helper for [`Encode`](trait.Encode.html), should be merged into [`Encoding`](trait.Encoding.html) but can't be due to HKT reasons.
pub trait Encoder<I: Iterator<Item = u8>>: Encoding {
	type Encoder: Iterator<Item = char>;
	fn encoder(self, iter: I) -> Self::Encoder;
}
/// Byte iterator adapter to an encoder.
///
/// Adapts any `Iterator<Item = u8>` into an iterator over the encoded chars.
///
/// Beware of code bloat! The entire decode logic may get inlined at the invocation site.
///
/// # Examples
///
/// ```
/// use basenc::Encode;
///
/// assert!(
/// 	"hello".bytes()
/// 	.encode(basenc::UpperHex)
/// 	.eq("68656C6C6F".chars())
/// );
///
/// assert!(
/// 	b"\xadapters\xff"[..].iter().cloned()
/// 	.encode(basenc::Base64Url)
/// 	.eq("rWFwdGVyc_8".chars())
/// );
///
/// assert!(
/// 	"STRingS".bytes()
/// 	.encode(basenc::LowerHex)
/// 	.eq("535452696e6753".chars())
/// );
/// ```
pub trait Encode<I: Iterator<Item = u8>, R: Encoder<I>> {
	fn encode(self, encoding: R) -> R::Encoder;
}
impl<I: Iterator<Item = u8>, R: Encoder<I>> Encode<I, R> for I {
	fn encode(self, encoding: R) -> R::Encoder {
		encoding.encoder(self)
	}
}

/// Create a decoder adapter for an encoding given an `Iterator<Item = char>`.
///
/// Helper for [`Decode`](trait.Decode.html), should be merged into [`Encoding`](trait.Encoding.html) but can't be due to HKT reasons.
pub trait Decoder<I: Iterator<Item = char>>: Encoding {
	type Decoder: Iterator<Item = Result<u8, Error>>;
	fn decoder(self, iter: I) -> Self::Decoder;
}
/// Char iterator adapter to a decoder.
///
/// Adapts any `Iterator<Item = char>` into an iterator over a result of the decoded bytes.
///
/// Beware of code bloat! The entire decode logic may get inlined at the invocation site.
///
/// # Examples
///
/// ```
/// use basenc::Decode;
///
/// assert!(
/// 	"68656c6c6F".chars()
/// 	.decode(basenc::AnyHex)
/// 	.eq("hello".bytes().map(Ok))
/// );
/// ```
pub trait Decode<I: Iterator<Item = char>, R: Decoder<I>> {
	fn decode(self, encoding: R) -> R::Decoder;
}
impl<I: Iterator<Item = char>, R: Decoder<I>> Decode<I, R> for I {
	fn decode(self, encoding: R) -> R::Decoder {
		encoding.decoder(self)
	}
}

//----------------------------------------------------------------

pub use details::base64::{Base64Std, Base64Url};
pub use details::hex::{LowerHex, UpperHex, AnyHex};
