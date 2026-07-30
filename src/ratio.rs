use core::num::NonZeroU8;

/// Ratio between encoded and decoded lengths.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ratio {
	/// Number of decoded bytes.
	pub decoded: NonZeroU8,
	/// Number of encoded bytes.
	pub encoded: NonZeroU8,
}

impl Ratio {
	/// Creates a ratio from non-zero decoded and encoded lengths.
	///
	/// # Panics
	///
	/// Panics if either length is zero.
	#[inline]
	pub const fn new(decoded: u8, encoded: u8) -> Self {
		let decoded = NonZeroU8::new(decoded).expect("decoded length must be non-zero");
		let encoded = NonZeroU8::new(encoded).expect("encoded length must be non-zero");
		Self { decoded, encoded }
	}

	/// Estimates the maximum length of the encoded data given the length of the decoded data.
	///
	/// # Panics
	///
	/// Panics if the result overflows `usize`.
	#[inline]
	pub const fn estimate_encoded_len(&self, len: usize) -> usize {
		if len == 0 {
			return 0;
		}
		let nchunks = (len - 1) / self.decoded.get() as usize + 1;
		match nchunks.checked_mul(self.encoded.get() as usize) {
			Some(len) => len,
			None => panic_overflow(),
		}
	}

	/// Estimates the maximum length of the decoded data given the length of the encoded data.
	///
	/// # Panics
	///
	/// Panics if the result overflows `usize`.
	#[inline]
	pub const fn estimate_decoded_len(&self, len: usize) -> usize {
		if len == 0 {
			return 0;
		}
		let nchunks = (len - 1) / self.encoded.get() as usize + 1;
		match nchunks.checked_mul(self.decoded.get() as usize) {
			Some(len) => len,
			None => panic_overflow(),
		}
	}

	/// Computes the chunk size for a given buffer length to incrementally encode the data.
	///
	/// The chunk size is always a multiple of the decoded length to ensure no padding is inserted.
	#[inline]
	pub const fn encoding_chunk_size(&self, buf_len: usize) -> usize {
		let units = buf_len / self.encoded.get() as usize;
		units * self.decoded.get() as usize
	}

	/// Computes the chunk size for a given buffer length to incrementally decode the data.
	///
	/// The chunk size is always a multiple of the encoded length to avoid partial decoding.
	#[inline]
	pub const fn decoding_chunk_size(&self, buf_len: usize) -> usize {
		let units = buf_len / self.decoded.get() as usize;
		units * self.encoded.get() as usize
	}
}

#[cold]
const fn panic_overflow() -> ! {
	panic!("overflow")
}

#[test]
fn test_ratio() {
	let ratio = Ratio::new(3, 4);

	assert_eq!(ratio.estimate_encoded_len(0), 0);
	assert_eq!(ratio.estimate_encoded_len(1), 4);
	assert_eq!(ratio.estimate_encoded_len(2), 4);
	assert_eq!(ratio.estimate_encoded_len(3), 4);
	assert_eq!(ratio.estimate_encoded_len(4), 8);
	assert_eq!(ratio.estimate_encoded_len(5), 8);
	assert_eq!(ratio.estimate_encoded_len(6), 8);

	assert_eq!(ratio.estimate_decoded_len(0), 0);
	assert_eq!(ratio.estimate_decoded_len(1), 3);
	assert_eq!(ratio.estimate_decoded_len(2), 3);
	assert_eq!(ratio.estimate_decoded_len(3), 3);
	assert_eq!(ratio.estimate_decoded_len(4), 3);
	assert_eq!(ratio.estimate_decoded_len(5), 6);
	assert_eq!(ratio.estimate_decoded_len(6), 6);
	assert_eq!(ratio.estimate_decoded_len(7), 6);

	assert_eq!(ratio.encoding_chunk_size(11), 6);
	assert_eq!(ratio.encoding_chunk_size(12), 9);
	assert_eq!(ratio.encoding_chunk_size(13), 9);
	assert_eq!(ratio.encoding_chunk_size(14), 9);
	assert_eq!(ratio.encoding_chunk_size(15), 9);
	assert_eq!(ratio.encoding_chunk_size(16), 12);

	assert_eq!(ratio.decoding_chunk_size(11), 12);
	assert_eq!(ratio.decoding_chunk_size(12), 16);
	assert_eq!(ratio.decoding_chunk_size(13), 16);
	assert_eq!(ratio.decoding_chunk_size(14), 16);
	assert_eq!(ratio.decoding_chunk_size(15), 20);
	assert_eq!(ratio.decoding_chunk_size(16), 20);
}

#[test]
#[should_panic(expected = "decoded length must be non-zero")]
fn ratio_rejects_zero_decoded_length() {
	Ratio::new(0, 1);
}

#[test]
#[should_panic(expected = "encoded length must be non-zero")]
fn ratio_rejects_zero_encoded_length() {
	Ratio::new(1, 0);
}
