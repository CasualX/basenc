use super::Encoding;

/// Padding policy.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Padding {
	/// Padding is not emitted while encoding and rejected while decoding.
	Forbidden,
	/// Padding is not emitted while encoding and accepted at the end while decoding.
	///
	/// Unpadded input is also accepted.
	Optional,
	/// Padding is emitted while encoding and required while decoding when needed.
	Required,
	/// Padding is not emitted while encoding and accepted between encoded segments while decoding.
	///
	/// Each segment must use canonical padding. Unpadded input and padding at the end are also accepted.
	Internal,
	/// Padding is emitted while encoding and accepted at the end while decoding.
	///
	/// Unpadded input is also accepted.
	#[default]
	Standard,
}

impl Padding {
	#[inline]
	pub(crate) const fn encode_padded(self) -> bool {
		matches!(self, Self::Standard | Self::Required)
	}

	#[inline]
	pub(crate) const fn decode_allows_padding(self) -> bool {
		!matches!(self, Self::Forbidden)
	}

	#[inline]
	pub(crate) const fn decode_requires_padding(self) -> bool {
		matches!(self, Self::Required)
	}

	#[inline]
	pub(crate) const fn decode_allows_internal_padding(self) -> bool {
		matches!(self, Self::Internal)
	}
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

	/// With a different padding policy.
	#[inline]
	pub const fn pad(&self, pad: Padding) -> WithPad<'a, T> {
		WithPad { encoding: self.encoding, pad }
	}
}

impl<T> WithPad<'_, T> where Self: Encoding {
	impl_encoding!();
}
