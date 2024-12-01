
/// Hex charset using lower-case letters.
///
/// Decoding accepts both lower- and upper-case letters.
#[derive(Clone, Debug)]
pub struct LowerHex;

impl crate::Encoding for LowerHex {
	#[inline]
	fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		encode(bytes, b'a', buffer)
	}

	#[inline]
	fn decode_into<B: crate::DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, crate::Error> {
		decode(string, buffer)
	}
}

impl_encoding!(LowerHex,
	encode: [
		"```",
		"assert_eq!(",
		"	basenc::LowerHex.encode(b\"\\x00\\x80\\xFF\\xDC\"),",
		"	\"0080ffdc\"",
		");",
		"```",
	],
	decode: [
		"```",
		"assert_eq!(",
		"	basenc::LowerHex.decode(\"0080FFdc\").unwrap(),",
		"	b\"\\x00\\x80\\xFF\\xDC\",",
		");",
	],
	encode_into: [
		"```",
		"let mut stack_buf = [0u8; 16];",
		"assert_eq!(",
		"	basenc::LowerHex.encode_into(b\"\\x00\\x80\\xFF\\xDC\", &mut stack_buf[..]),",
		"	\"0080ffdc\"",
		");",
		"```",
	],
	decode_into: [
		"```",
		"assert_eq!(",
		"	basenc::LowerHex.decode_into(\"0080FFdc\", Vec::new()).unwrap(),",
		"	b\"\\x00\\x80\\xFF\\xDC\",",
		");",
		"```",
	],
);

/// Hex charset using upper-case letters.
///
/// Decoding accepts both lower- and upper-case letters.
#[derive(Clone, Debug)]
pub struct UpperHex;

impl crate::Encoding for UpperHex {
	#[inline]
	fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		encode(bytes, b'A', buffer)
	}

	#[inline]
	fn decode_into<B: crate::DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, crate::Error> {
		decode(string, buffer)
	}
}

impl_encoding!(UpperHex,
	encode: [
		"```",
		"assert_eq!(",
		"	basenc::UpperHex.encode(b\"\\x00\\x80\\xFF\\xDC\"),",
		"	\"0080FFDC\"",
		");",
		"```",
	],
	decode: [
		"```",
		"assert_eq!(",
		"	basenc::UpperHex.decode(\"0080ffDC\").unwrap(),",
		"	b\"\\x00\\x80\\xFF\\xDC\",",
		");",
		"```",
	],
	encode_into: [
		"```",
		"let mut stack_buf = [0u8; 16];",
		"assert_eq!(",
		"	basenc::UpperHex.encode_into(b\"\\x00\\x80\\xFF\\xDC\", &mut stack_buf[..]),",
		"	\"0080FFDC\"",
		");",
		"```",
	],
	decode_into: [
		"```",
		"assert_eq!(",
		"	basenc::UpperHex.decode_into(\"0080ffDC\", Vec::new()).unwrap(),",
		"	b\"\\x00\\x80\\xFF\\xDC\",",
		");",
		"```",
	],
);

//----------------------------------------------------------------
// Encoding

mod encode;

#[inline(never)]
fn encode<B: crate::EncodeBuf>(bytes: &[u8], base: u8, mut buffer: B) -> B::Output {
	// Hex encodes each byte into two chars
	let encode_len = match usize::checked_add(bytes.len(), bytes.len()) {
		Some(len) => len,
		None => crate::panic_overflow(),
	};

	unsafe {
		let buf = buffer.allocate(encode_len);
		encode::encode(bytes.as_ptr(), bytes.len(), buf, base);
		buffer.commit(encode_len)
	}
}

//----------------------------------------------------------------
// Decoding

mod decode;

#[inline(never)]
fn decode<B: crate::DecodeBuf>(string: &str, mut buffer: B) -> Result<B::Output, crate::Error> {
	let string = string.as_bytes();

	if string.len() % 2 != 0 {
		return Err(crate::Error::BadLength);
	}

	let decode_len = string.len() / 2;

	unsafe {
		let buf = buffer.allocate(decode_len);
		decode::decode(string.as_ptr(), decode_len, buf)?;
		Ok(buffer.commit(decode_len))
	}
}
