use super::Encoding;

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

/// Encoding with explicit padding policy.
#[derive(Clone, Debug)]
pub struct WithPad<'a, T> {
	pub(crate) encoding: &'a T,
	pub(crate) pad: Padding,
}

impl<'a, T> WithPad<'a, T> {
	/// Constructor.
	#[inline]
	pub const fn new(encoding: &'a T, pad: Padding) -> Self {
		WithPad { encoding, pad }
	}
}

impl<T> WithPad<'_, T> where Self: Encoding {
	#[cfg(feature = "std")]
	/// Encodes the input bytes.
	#[inline]
	pub fn encode(&self, bytes: &[u8]) -> std::string::String {
		crate::Encoding::encode_into(self, bytes, std::string::String::new())
	}

	#[cfg(feature = "std")]
	/// Decodes the input string.
	#[inline]
	pub fn decode(&self, string: &str) -> Result<std::vec::Vec<u8>, crate::Error> {
		crate::Encoding::decode_into(self, string.as_bytes(), std::vec::Vec::new())
	}

	/// Encodes into a buffer.
	#[inline]
	pub fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		crate::Encoding::encode_into(self, bytes, buffer)
	}

	/// Decodes into a buffer.
	#[inline]
	pub fn decode_into<B: crate::DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, crate::Error> {
		crate::Encoding::decode_into(self, string.as_bytes(), buffer)
	}

	/// Wraps the encoding and bytes for display.
	#[inline]
	pub fn display<'a>(&'a self, bytes: &'a [u8]) -> crate::Display<'a, Self> {
		crate::Display::new(self, bytes)
	}
}
