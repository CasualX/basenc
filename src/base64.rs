use crate::Padding;

const PAD_CHAR: u8 = b'=';

/// Base64 alphabet.
#[derive(Clone, Debug)]
pub struct Base64 {
	/// Character set.
	///
	/// Must contain only ASCII characters.
	charset: [u8; 64],
	/// Lookup table.
	///
	/// Maps ASCII characters to their index in the charset.
	/// Invalid characters are mapped to 255.
	lut: [u8; 127],
	/// If padding is required.
	padding: Padding,
}

impl Base64 {
	/// Creates a new Base64 alphabet.
	///
	/// # Panics
	///
	/// Panics if the alphabet contains `=`, duplicate or non-ASCII characters.
	pub const fn new(&charset: &[u8; 64], padding: Padding) -> Self {
		let mut lut = [255; 127];
		let mut i = 0;
		while i < charset.len() {
			if charset[i] == PAD_CHAR {
				panic!("padding character in Base64 charset");
			}
			if charset[i] >= 127 {
				panic!("non-ASCII character in Base64 charset");
			}
			if lut[charset[i] as usize] != 255 {
				panic!("duplicate character in Base64 charset");
			}
			lut[charset[i] as usize] = i as u8;
			i += 1;
		}
		Base64 { charset, lut, padding }
	}
}

impl crate::Encoding for Base64 {
	#[inline]
	fn encode_into<B: crate::EncodeBuf>(&self, bytes: &[u8], buffer: B) -> B::Output {
		encode(bytes, self, buffer)
	}

	#[inline]
	fn decode_into<B: crate::DecodeBuf>(&self, string: &str, buffer: B) -> Result<B::Output, crate::Error> {
		decode(string, self, buffer)
	}
}

impl_encoding!(Base64,
	encode: [
		"```",
		"assert_eq!(",
		"	basenc::Base64Std.encode(b\"hello world\"),",
		"	\"aGVsbG8gd29ybGQ\"",
		");",
		"```",
	],
	decode: [
		"```",
		"assert_eq!(",
		"	basenc::Base64Std.decode(\"aGVsbG8gd29ybGQ=\").unwrap(),",
		"	b\"hello world\",",
		");",
		"```",
	],
	encode_into: [
		"```",
		"let mut stack_buf = [0u8; 16];",
		"assert_eq!(",
		"	basenc::Base64Std.encode_into(b\"hello world\", &mut stack_buf[..]),",
		"	\"aGVsbG8gd29ybGQ\"",
		");",
		"```",
	],
	decode_into: [
		"```",
		"let mut buffer = vec![0x11, 0x22, 0x33];",
		"assert_eq!(",
		"	basenc::Base64Url.decode_into(\"QnVGZkVyIFJlVXNFIQ\", &mut buffer).unwrap(),",
		"	b\"BuFfEr ReUsE!\",",
		");",
		"assert_eq!(buffer, b\"\\x11\\x22\\x33BuFfEr ReUsE!\");",
		"```",
	],
);

/// Base64 standard charset.
///
/// Alphabet is `A-Za-z0-9+/` with optional `=` padding.
#[allow(non_upper_case_globals)]
pub static Base64Std: Base64 = Base64::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/", Padding::Optional);

/// Base64 standard charset with strict padding.
///
/// Alphabet is `A-Za-z0-9+/` with strict `=` padding.
#[allow(non_upper_case_globals)]
pub static Base64StdStrict: Base64 = Base64::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/", Padding::Strict);

/// Base64 standard charset without padding.
///
/// Alphabet is `A-Za-z0-9+/` without padding.
#[allow(non_upper_case_globals)]
pub static Base64StdNoPad: Base64 = Base64::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/", Padding::None);

/// Base64 url-safe charset.
///
/// Alphabet is `A-Za-z0-9-_` with optional padding.
#[allow(non_upper_case_globals)]
pub static Base64Url: Base64 = Base64::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_", Padding::Optional);

//----------------------------------------------------------------
// Encoding

mod encode;

fn encode_len(len: usize, pad: Padding) -> usize {
	let a = len / 3;
	let b = len % 3;
	let extra = if matches!(pad, Padding::Strict) {
		if b == 0 { 0 } else { 4 }
	}
	else {
		match b {
			0 => 0,
			1 => 2,
			2 => 3,
			_ => unreachable!(),
		}
	};
	match a.checked_mul(4).and_then(|x| x.checked_add(extra)) {
		Some(x) => x,
		None => crate::panic_overflow(),
	}
}

#[inline(never)]
pub fn encode<B: crate::EncodeBuf>(bytes: &[u8], base: &Base64, mut buffer: B) -> B::Output {
	// Base64 encodes each 3 bytes into 4 chars
	let encode_len = encode_len(bytes.len(), base.padding);

	unsafe {
		let buf = buffer.allocate(encode_len);
		encode::encode(bytes, base, buf);
		buffer.commit(encode_len)
	}
}

//----------------------------------------------------------------
// Decoding

mod decode;

fn decode_len(string: &[u8], base: &Base64) -> Result<usize, crate::Error> {
	if string.len() == 0 {
		return Ok(0);
	}

	match base.padding {
		Padding::Strict => decode_len_strict(string),
		Padding::Optional => decode_len_optional(string),
		Padding::None => decode_len_none(string),
	}
}

fn decode_len_strict(string: &[u8]) -> Result<usize, crate::Error> {
	if string.len() == 0 {
		return Ok(0);
	}

	if string.len() % 4 != 0 {
		return Err(crate::Error::BadLength);
	}

	let mut len = string.len() / 4 * 3;
	if string[string.len() - 2] == PAD_CHAR && string[string.len() - 1] == PAD_CHAR {
		len -= 2;
	}
	else if string[string.len() - 1] == PAD_CHAR {
		len -= 1;
	}

	Ok(len)
}

fn decode_len_optional(string: &[u8]) -> Result<usize, crate::Error> {
	if string.len() == 0 {
		return Ok(0);
	}

	let mut len = string.len() / 4 * 3;
	match string.len() % 4 {
		1 => return Err(crate::Error::BadLength),
		2 => len += 1,
		3 => len += 2,
		_ => if string[string.len() - 2] == PAD_CHAR && string[string.len() - 1] == PAD_CHAR {
			len -= 2;
		}
		else if string[string.len() - 1] == PAD_CHAR {
			len -= 1;
		},
	};

	Ok(len)
}

fn decode_len_none(string: &[u8]) -> Result<usize, crate::Error> {
	if string.len() == 0 {
		return Ok(0);
	}

	let mut len = string.len() / 4 * 3;
	len += match string.len() % 4 {
		1 => return Err(crate::Error::BadLength),
		2 => 1,
		3 => 2,
		_ => 0,
	};

	Ok(len)
}

#[inline(never)]
pub fn decode<B: crate::DecodeBuf>(string: &str, base: &Base64, mut buffer: B) -> Result<B::Output, crate::Error> {
	let decode_len = decode_len(string.as_bytes(), base)?;

	unsafe {
		let buf = buffer.allocate(decode_len);
		decode::decode(string.as_bytes(), base, buf)?;
		Ok(buffer.commit(decode_len))
	}
}
