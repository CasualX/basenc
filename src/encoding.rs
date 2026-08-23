
macro_rules! impl_encoding {
	(
		$(
			encode: [$(#[doc = $encode_example:literal])*],
			decode: [$(#[doc = $decode_example:literal])*],
			encode_into: [$(#[doc = $encode_into_example:literal])*],
			decode_into: [$(#[doc = $decode_into_example:literal])*],
		)?
	) => {
		#[cfg(feature = "alloc")]
		/// Encodes the input bytes.
		$(
			///
			/// # Examples
			$(#[doc = $encode_example])*
		)?
		#[inline]
		pub fn encode(&self, bytes: &[u8]) -> alloc::string::String {
			crate::Encoding::encode_bytes_into(self, bytes, alloc::string::String::new())
		}

		#[cfg(feature = "alloc")]
		/// Decodes the input string.
		$(
			///
			/// # Examples
			$(#[doc = $decode_example])*
		)?
		#[inline]
		pub fn decode(&self, string: &str) -> Result<alloc::vec::Vec<u8>, crate::Error> {
			self.decode_bytes(string.as_bytes())
		}

		#[cfg(feature = "alloc")]
		/// Decodes the input bytes.
		#[inline]
		pub fn decode_bytes(&self, bytes: &[u8]) -> Result<alloc::vec::Vec<u8>, crate::Error> {
			crate::Encoding::decode_bytes_into(self, bytes, alloc::vec::Vec::new())
		}

		/// Encodes into a buffer.
		$(
			///
			/// # Examples
			$(#[doc = $encode_into_example])*
		)?
		#[inline]
		pub fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
			crate::Encoding::encode_bytes_into(self, bytes, buffer)
		}

		/// Decodes into a buffer.
		$(
			///
			/// # Examples
			$(#[doc = $decode_into_example])*
		)?
		#[inline]
		pub fn decode_into<B: crate::DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, crate::Error> {
			self.decode_bytes_into(string.as_bytes(), buffer)
		}

		/// Decodes bytes into a buffer.
		#[inline]
		pub fn decode_bytes_into<B: crate::DecodeBuf>(&self, bytes: &[u8], buffer: B) -> Result<B::Output, crate::Error> {
			crate::Encoding::decode_bytes_into(self, bytes, buffer)
		}

		/// Wraps the encoding and bytes for display.
		#[inline]
		pub fn display<'a>(&'a self, bytes: &'a [u8]) -> crate::Display<'a, Self> {
			crate::Display::new(self, bytes)
		}
	};
}
