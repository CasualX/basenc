use super::*;

const RATIO: Ratio = Ratio { decoded: 3, encoded: 4 };
const PAD_CHAR: u8 = b'=';

/// Base64 alphabet.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Base64 {
	/// Character set.
	///
	/// Must contain only ASCII characters.
	charset: [u8; 64],
	/// Lookup table.
	///
	/// Maps ASCII characters to their index in the charset.
	/// Invalid characters are mapped to 255.
	lut: [u8; 128],
}

impl Base64 {
	/// Creates a new Base64 alphabet.
	///
	/// # Panics
	///
	/// Panics if the alphabet contains `=`, duplicate or non-ASCII characters.
	const fn from_charset(&charset: &[u8; 64]) -> Self {
		let mut lut = [255; 128];
		let mut i = 0;
		while i < charset.len() {
			if charset[i] == PAD_CHAR {
				panic!("padding character in Base64 charset");
			}
			if charset[i] as usize >= lut.len() {
				panic!("non-ASCII character in Base64 charset");
			}
			if lut[charset[i] as usize] != 255 {
				panic!("duplicate character in Base64 charset");
			}
			lut[charset[i] as usize] = i as u8;
			i += 1;
		}
		Base64 { charset, lut }
	}

	/// Creates a new Base64 alphabet.
	///
	/// For optimization purposes, the alphabet always starts with `A-Za-z0-9`.
	/// The last two characters can be customized.
	///
	/// # Panics
	///
	/// Panics if the alphabet contains `=`, duplicate or non-ASCII characters.
	pub const fn new(
		char62: u8,
		char63: u8,
	) -> Base64 {
		let charset_upper_index = 0;
		let charset_lower_index = 26;
		let charset_digits_index = 52;
		assert!(char62 < 128 && char62 != PAD_CHAR);
		assert!(char63 < 128 && char63 != PAD_CHAR);
		let mut charset = [0; 64];
		let mut i;
		i = 0;
		while i < 26 {
			charset[(charset_upper_index + i) as usize] = b'A' + i;
			i += 1;
		}
		i = 0;
		while i < 26 {
			charset[(charset_lower_index + i) as usize] = b'a' + i;
			i += 1;
		}
		i = 0;
		while i < 10 {
			charset[(charset_digits_index + i) as usize] = b'0' + i;
			i += 1;
		}
		charset[62] = char62;
		charset[63] = char63;
		Base64::from_charset(&charset)
	}

	/// With explicit padding policy.
	pub const fn pad(&self, pad: Padding) -> WithPad<'_, Self> {
		WithPad { encoding: self, pad }
	}
}

impl Encoding for Base64 {
	const RATIO: Ratio = RATIO;

	#[inline]
	fn encode_into<B: EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		encode(bytes, self, Padding::Optional, buffer)
	}

	#[inline]
	fn decode_into<B: DecodeBuf>(&self, string: &[u8], buffer: B) -> Result<B::Output, Error> {
		decode(string, self, Padding::Optional, buffer)
	}
}

impl Encoding for WithPad<'_, Base64> {
	const RATIO: Ratio = RATIO;

	#[inline]
	fn encode_into<B: EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		encode(bytes, self.encoding, self.pad, buffer)
	}

	#[inline]
	fn decode_into<B: DecodeBuf>(&self, string: &[u8], buffer: B) -> Result<B::Output, Error> {
		decode(string, self.encoding, self.pad, buffer)
	}
}

impl_encoding!(Base64,
	encode: [
		"```",
		"let encoded = basenc::Base64Std.encode(b\"hello world\");",
		"assert_eq!(encoded, \"aGVsbG8gd29ybGQ\");",
		"```",
	],
	decode: [
		"```",
		"let decoded = basenc::Base64Std.decode(\"aGVsbG8gd29ybGQ=\").unwrap();",
		"assert_eq!(decoded, b\"hello world\");",
		"```",
	],
	encode_into: [
		"```",
		"let mut stack_buf = [0u8; 16];",
		"let encoded = basenc::Base64Std.encode_into(b\"hello world\", &mut stack_buf);",
		"assert_eq!(encoded, \"aGVsbG8gd29ybGQ\");",
		"```",
	],
	decode_into: [
		"```",
		"let mut buffer = vec![0x11, 0x22, 0x33];",
		"let decoded = basenc::Base64Url.decode_into(\"QnVGZkVyIFJlVXNFIQ\", &mut buffer).unwrap();",
		"assert_eq!(decoded, b\"BuFfEr ReUsE!\");",
		"assert_eq!(buffer, b\"\\x11\\x22\\x33BuFfEr ReUsE!\");",
		"```",
	],
);

//----------------------------------------------------------------

/// Base64 standard charset.
///
/// The alphabet is `A-Za-z0-9+/`.
#[allow(non_upper_case_globals)]
pub static Base64Std: Base64 = Base64::new(b'+', b'/');

/// Base64 url-safe charset.
///
/// The alphabet is `A-Za-z0-9-_`.
#[allow(non_upper_case_globals)]
pub static Base64Url: Base64 = Base64::new(b'-', b'_');

//----------------------------------------------------------------
// Encoding

mod encode;

#[inline(never)]
fn encode<B: EncodeBuf>(bytes: &[u8], base: &Base64, pad: Padding, mut buffer: B) -> B::Output {
	let dest_len = RATIO.estimate_encoded_len(bytes.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = encode::encode_fn()(bytes, base, pad, dest);
		let len = end.offset_from(dest) as usize;
		buffer.commit(len)
	}
}

//----------------------------------------------------------------
// Decoding

mod decode;

#[inline(never)]
fn decode<B: DecodeBuf>(string: &[u8], base: &Base64, pad: Padding, mut buffer: B) -> Result<B::Output, Error> {
	let dest_len = RATIO.estimate_decoded_len(string.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = decode::decode_fn()(string, base, pad, dest)?;
		let len = end.offset_from(dest) as usize;
		Ok(buffer.commit(len))
	}
}
