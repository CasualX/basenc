
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
			pub fn encode(&self, bytes: &[u8], pad: Padding) -> std::string::String {
				crate::Encoding::encode_into(self, bytes, pad, std::string::String::new())
			}

			#[cfg(feature = "std")]
			/// Decodes the input string.
			$(
				///
				/// # Examples
				$(#[doc = $decode_example])*
			)?
			#[inline]
			pub fn decode(&self, string: &str, pad: Padding) -> Result<std::vec::Vec<u8>, crate::Error> {
				crate::Encoding::decode_into(self, string.as_bytes(), pad, std::vec::Vec::new())
			}

			/// Encodes into a buffer.
			$(
				///
				/// # Examples
				$(#[doc = $encode_into_example])*
			)?
			#[inline]
			pub fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], pad: Padding, buffer: B) -> B::Output {
				crate::Encoding::encode_into(self, bytes, pad, buffer)
			}

			/// Decodes into a buffer.
			$(
				///
				/// # Examples
				$(#[doc = $decode_into_example])*
			)?
			#[inline]
			pub fn decode_into<B: crate::DecodeBuf>(&self, string: &str, pad: Padding, buffer: B) -> Result<B::Output, crate::Error> {
				crate::Encoding::decode_into(self, string.as_bytes(), pad, buffer)
			}

			/// Wraps the encoding and bytes for display.
			#[inline]
			pub fn display<'a>(&'a self, bytes: &'a [u8], pad: Padding) -> crate::Display<'a, Self> {
				crate::Display::new(self, bytes, pad)
			}
		}
	};
}

macro_rules! impl_encoding_no_pad {
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
				crate::Encoding::encode_into(self, bytes, crate::NoPad, std::string::String::new())
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
				crate::Encoding::decode_into(self, string.as_bytes(), crate::NoPad, std::vec::Vec::new())
			}

			/// Encodes into a buffer.
			$(
				///
				/// # Examples
				$(#[doc = $encode_into_example])*
			)?
			#[inline]
			pub fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
				crate::Encoding::encode_into(self, bytes, crate::NoPad, buffer)
			}

			/// Decodes into a buffer.
			$(
				///
				/// # Examples
				$(#[doc = $decode_into_example])*
			)?
			#[inline]
			pub fn decode_into<B: crate::DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, crate::Error> {
				crate::Encoding::decode_into(self, string.as_bytes(), crate::NoPad, buffer)
			}

			/// Wraps the encoding and bytes for display.
			#[inline]
			pub fn display<'a>(&'a self, bytes: &'a [u8]) -> crate::Display<'a, Self> {
				crate::Display::new(self, bytes, crate::NoPad)
			}
		}
	};
}
