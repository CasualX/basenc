use super::Encoding;

/// Padding policy.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Padding {
	/// Padding is not emitted while encoding and rejected while decoding.
	Forbidden,
	/// Padding is not emitted while encoding and accepted at the end while decoding.
	///
	/// Unpadded input is also accepted.
	#[default]
	Optional,
	/// Padding is emitted while encoding and required while decoding when needed.
	Required,
	/// Padding is not emitted while encoding and accepted between encoded segments while decoding.
	///
	/// Each segment must use canonical padding. Unpadded input and padding at the end are also accepted.
	Internal,
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
	impl_encoding!();
}
