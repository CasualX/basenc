use super::*;

const RATIO: Ratio = Ratio::new(3, 4);
const PAD_CHAR: u8 = b'=';
const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const LUT: [u8; 128] = make_lut();

const fn make_lut() -> [u8; 128] {
	let mut lut = [255; 128];
	let mut index = 0;
	while index < CHARSET.len() {
		lut[CHARSET[index] as usize] = index as u8;
		index += 1;
	}
	lut
}

/// Base64 standard encoding.
///
/// The alphabet is `A-Za-z0-9+/`.
#[derive(Copy, Clone, Debug, Default)]
pub struct Base64Std;

impl Base64Std {
	/// With explicit padding policy.
	pub const fn pad(&self, pad: Padding) -> WithPad<'_, Self> {
		WithPad { encoding: self, pad }
	}
}

impl Sealed for Base64Std {}

impl Encoding for Base64Std {
	const RATIO: Ratio = RATIO;

	#[inline]
	fn encode_bytes_into<B: EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		encode(bytes, Padding::Standard, buffer)
	}

	#[inline]
	fn decode_bytes_into<B: DecodeBuf>(&self, string: &[u8], buffer: B) -> Result<B::Output, Error> {
		decode(string, Padding::Standard, buffer)
	}
}

impl Sealed for WithPad<'_, Base64Std> {}

impl Encoding for WithPad<'_, Base64Std> {
	const RATIO: Ratio = RATIO;

	#[inline]
	fn encode_bytes_into<B: EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		encode(bytes, self.pad, buffer)
	}

	#[inline]
	fn decode_bytes_into<B: DecodeBuf>(&self, string: &[u8], buffer: B) -> Result<B::Output, Error> {
		decode(string, self.pad, buffer)
	}
}

impl Base64Std {
	impl_encoding!(
		encode: [
			/// ```
			/// let encoded = basenc::Base64Std.encode(b"hello world");
			/// assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
			/// ```
		],
		decode: [
			/// ```
			/// let decoded = basenc::Base64Std.decode("aGVsbG8gd29ybGQ=").unwrap();
			/// assert_eq!(decoded, b"hello world");
			/// ```
		],
		encode_into: [
			/// ```
			/// let mut stack_buf = [0u8; 16];
			/// let encoded = basenc::Base64Std.encode_into(b"hello world", &mut stack_buf);
			/// assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
			/// ```
		],
		decode_into: [
			/// ```
			/// let mut stack_buf = [0u8; 16];
			/// let decoded = basenc::Base64Std.decode_into("QnVGZkVyIFJlVXNFIQ", &mut stack_buf).unwrap();
			/// assert_eq!(decoded, b"BuFfEr ReUsE!");
			/// ```
		],
	);
}

mod encode;

#[inline(never)]
fn encode<B: EncodeBuf>(bytes: &[u8], pad: Padding, mut buffer: B) -> B::Output {
	let dest_len = RATIO.estimate_encoded_len(bytes.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = encode::encode_fn()(bytes, pad, dest);
		let len = end.offset_from(dest) as usize;
		buffer.commit(len)
	}
}

mod decode;

#[inline(never)]
fn decode<B: DecodeBuf>(string: &[u8], pad: Padding, mut buffer: B) -> Result<B::Output, Error> {
	let dest_len = RATIO.estimate_decoded_len(string.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = decode::decode_fn()(string, pad, dest)?;
		let len = end.offset_from(dest) as usize;
		Ok(buffer.commit(len))
	}
}
