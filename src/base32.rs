use super::*;

const RATIO: Ratio = Ratio { decoded: 5, encoded: 8 };
const PAD_CHAR: u8 = b'=';

/// Base32 alphabet.
#[derive(Clone, Debug)]
pub struct Base32 {
	charset: [u8; 32],
	lut: [u8; 128],
}

impl Base32 {
	/// Creates a new Base32 alphabet.
	///
	/// # Panics
	///
	/// Panics if the alphabet contains `=`, duplicate or non-ASCII characters.
	pub const fn new(&charset: &[u8; 32]) -> Self {
		let mut lut = [255; 128];
		let mut i = 0;
		while i < charset.len() {
			if charset[i] == PAD_CHAR {
				panic!("padding character in Base32 charset");
			}
			if charset[i] as usize >= lut.len() {
				panic!("non-ASCII character in Base32 charset");
			}
			if lut[charset[i] as usize] != 255 {
				panic!("duplicate character in Base32 charset");
			}
			lut[charset[i] as usize] = i as u8;
			i += 1;
		}
		Base32 { charset, lut }
	}
}

impl Encoding for Base32 {
	const RATIO: Ratio = RATIO;

	#[inline]
	fn encode_into<B: EncodeBuf>(&self, bytes: &[u8], pad: Padding, buffer: B) -> B::Output {
		encode(bytes, self, pad, buffer)
	}

	#[inline]
	fn decode_into<B: DecodeBuf>(&self, string: &[u8], pad: Padding, buffer: B) -> Result<B::Output, Error> {
		decode(string, self, pad, buffer)
	}
}

impl_encoding!(Base32);

//----------------------------------------------------------------

/// Base32 RFC 4648 alphabet.
///
/// The alphabet is `A-Z2-7`.
#[allow(non_upper_case_globals)]
pub static Base32Std: Base32 = Base32::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");

/// Base32 Extended Hex alphabet.
///
/// The alphabet is `0-9A-V`.
#[allow(non_upper_case_globals)]
pub static Base32Hex: Base32 = Base32::new(b"0123456789ABCDEFGHIJKLMNOPQRSTUV");

/// z-base-32 alphabet.
///
/// The alphabet is `ybndrfg8ejkmcpqxot1uwisza345h769`.
#[allow(non_upper_case_globals)]
pub static Base32Z: Base32 = Base32::new(b"ybndrfg8ejkmcpqxot1uwisza345h769");

//----------------------------------------------------------------
// Encoding

mod encode;

#[inline(never)]
fn encode<B: EncodeBuf>(bytes: &[u8], base: &Base32, pad: Padding, mut buffer: B) -> B::Output {
	let dest_len = RATIO.estimate_encoded_len(bytes.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = encode::encode(bytes, base, pad, dest);
		let len = end.offset_from(dest) as usize;
		buffer.commit(len)
	}
}

//----------------------------------------------------------------
// Decoding

mod decode;

#[inline(never)]
fn decode<B: DecodeBuf>(string: &[u8], base: &Base32, pad: Padding, mut buffer: B) -> Result<B::Output, Error> {
	let dest_len = RATIO.estimate_decoded_len(string.len());

	unsafe {
		let dest = buffer.allocate(dest_len);
		let end = decode::decode(string, base, pad, dest)?;
		let len = end.offset_from(dest) as usize;
		Ok(buffer.commit(len))
	}
}
