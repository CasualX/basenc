use super::*;

const RATIO: Ratio = Ratio { decoded: 1, encoded: 2 };

/// Hex charset using lower-case letters.
///
/// Decoding accepts both lower- and upper-case letters.
#[derive(Clone, Debug)]
pub struct LowerHex;

impl Encoding for LowerHex {
	const RATIO: Ratio = RATIO;

	#[inline]
	fn encode_into<B: EncodeBuf>(&self, bytes: &[u8], _pad: Padding, buffer: B) -> B::Output {
		encode(bytes, b'a', buffer)
	}

	#[inline]
	fn decode_into<B: DecodeBuf>(&self, string: &[u8], _pad: Padding, buffer: B) -> Result<B::Output, Error> {
		decode(string, buffer)
	}
}

impl_encoding_no_pad!(LowerHex,
	encode: [
		"```",
		"let encoded = basenc::LowerHex.encode(b\"\\x00\\x80\\xFF\\xDC\");",
		"assert_eq!(encoded, \"0080ffdc\");",
		"```",
	],
	decode: [
		"```",
		"let decoded = basenc::LowerHex.decode(\"0080FFdc\").unwrap();",
		"assert_eq!(decoded, b\"\\x00\\x80\\xFF\\xDC\");",
	],
	encode_into: [
		"```",
		"let mut stack_buf = [0u8; 16];",
		"let encoded = basenc::LowerHex.encode_into(b\"\\x00\\x80\\xFF\\xDC\", &mut stack_buf);",
		"assert_eq!(encoded, \"0080ffdc\");",
		"```",
	],
	decode_into: [
		"```",
		"let decoded = basenc::LowerHex.decode_into(\"0080FFdc\", Vec::new()).unwrap();",
		"assert_eq!(decoded, b\"\\x00\\x80\\xFF\\xDC\");",
		"```",
	],
);

//----------------------------------------------------------------

/// Hex charset using upper-case letters.
///
/// Decoding accepts both lower- and upper-case letters.
#[derive(Clone, Debug)]
pub struct UpperHex;

impl Encoding for UpperHex {
	const RATIO: Ratio = RATIO;

	#[inline]
	fn encode_into<B: EncodeBuf>(&self, bytes: &[u8], _pad: Padding, buffer: B) -> B::Output {
		encode(bytes, b'A', buffer)
	}

	#[inline]
	fn decode_into<B: DecodeBuf>(&self, string: &[u8], _pad: Padding, buffer: B) -> Result<B::Output, Error> {
		decode(string, buffer)
	}
}

impl_encoding_no_pad!(UpperHex,
	encode: [
		"```",
		"let encoded = basenc::UpperHex.encode(b\"\\x00\\x80\\xFF\\xDC\");",
		"assert_eq!(encoded, \"0080FFDC\");",
		"```",
	],
	decode: [
		"```",
		"let decoded = basenc::UpperHex.decode(\"0080ffDC\").unwrap();",
		"assert_eq!(decoded, b\"\\x00\\x80\\xFF\\xDC\");",
		"```",
	],
	encode_into: [
		"```",
		"let mut stack_buf = [0u8; 16];",
		"let encoded = basenc::UpperHex.encode_into(b\"\\x00\\x80\\xFF\\xDC\", &mut stack_buf);",
		"assert_eq!(encoded, \"0080FFDC\");",
		"```",
	],
	decode_into: [
		"```",
		"let decoded = basenc::UpperHex.decode_into(\"0080ffDC\", Vec::new()).unwrap();",
		"assert_eq!(decoded, b\"\\x00\\x80\\xFF\\xDC\");",
		"```",
	],
);

//----------------------------------------------------------------
// Encoding

mod encode;

#[inline(never)]
fn encode<B: EncodeBuf>(bytes: &[u8], base: u8, mut buffer: B) -> B::Output {
	let dest_len = RATIO.estimate_encoded_len(bytes.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = encode::encode_fn()(bytes, dest, base);
		let len = end.offset_from(dest) as usize;
		buffer.commit(len)
	}
}

//----------------------------------------------------------------
// Decoding

mod decode;

#[inline(never)]
fn decode<B: DecodeBuf>(string: &[u8], mut buffer: B) -> Result<B::Output, Error> {
	let dest_len = RATIO.estimate_decoded_len(string.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = decode::decode_fn()(string, dest)?;
		let len = end.offset_from(dest) as usize;
		Ok(buffer.commit(len))
	}
}
