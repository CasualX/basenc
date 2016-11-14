/*!
Hex codec.
*/

pub mod encode;
pub mod decode;

//----------------------------------------------------------------

/// Hex charset.
///
/// # Safety
///
/// The characters in the alphabet must be ASCII.
pub unsafe trait CharSet: Copy {
	/// Lookup table for index to char.
	fn chars(self) -> &'static str;
	/// Lookup table for char to index.
	fn lut(self) -> (char, &'static [u8]);
	/// Encodes an index in the alphabet as a character.
	///
	/// Returns an error if the index is out of range.
	fn encode_char(self, index: u8) -> Result<char, u8> {
		self.chars().as_bytes().get(index as usize).map(|&byte| byte as char).ok_or(index)
	}
	/// Encodes a character as an index in the alphabet.
	///
	/// Returns an error if the character is not valid for the alphabet.
	fn decode_char(self, chr: char) -> Result<u8, char> {
		let (base, lut) = self.lut();
		let offset = (chr as u32).wrapping_sub(base as u32) as usize;
		if let Some(&index) = lut.get(offset) {
			if index == 255 { Err(chr) }
			else { Ok(index) }
		}
		else { Err(chr) }
	}
}

//----------------------------------------------------------------

/// Hex charset using strictly lower-case letters.
#[derive(Copy, Clone, Debug)]
pub struct LowerHex;
unsafe impl CharSet for LowerHex {
	fn chars(self) -> &'static str {
		"0123456789abcdef"
	}
	fn lut(self) -> (char, &'static [u8]) {
		static MAP: [u8; 55] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 10, 11, 12, 13, 14, 15];
		(48 as char, &MAP)
	}
	// #[cfg(not(feature = "lut"))]
	fn encode_char(self, index: u8) -> Result<char, u8> {
		match index {
			0...9 => Ok((index + b'0') as char),
			10...15 => Ok((index - 10 + b'a') as char),
			_ => Err(index),
		}
	}
	#[cfg(not(feature = "lut"))]
	fn decode_char(self, chr: char) -> Result<u8, char> {
		match chr {
			'0'...'9' => Ok(chr as u8 - b'0'),
			'a'...'f' => Ok(chr as u8 - b'a' + 10),
			_ => Err(chr),
		}
	}
}

//----------------------------------------------------------------

/// Hex charset using strictly upper-case letters.
#[derive(Copy, Clone, Debug)]
pub struct UpperHex;
unsafe impl CharSet for UpperHex {
	fn chars(self) -> &'static str {
		"0123456789ABCDEF"
	}
	fn lut(self) -> (char, &'static [u8]) {
		static MAP: [u8; 23] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 255, 255, 255, 255, 10, 11, 12, 13, 14, 15];
		(48 as char, &MAP)
	}
	// #[cfg(not(feature = "lut"))]
	fn encode_char(self, index: u8) -> Result<char, u8> {
		match index {
			0...9 => Ok((index + b'0') as char),
			10...15 => Ok((index - 10 + b'A') as char),
			_ => Err(index),
		}
	}
	#[cfg(not(feature = "lut"))]
	fn decode_char(self, chr: char) -> Result<u8, char> {
		match chr {
			'0'...'9' => Ok(chr as u8 - b'0'),
			'A'...'F' => Ok(chr as u8 - b'A' + 10),
			_ => Err(chr),
		}
	}
}

//----------------------------------------------------------------

/// Hex charset using lower-case letters.
///
/// In the spirit of the robustness principle, decoding will accept both lower- and upper-case letters.
#[derive(Copy, Clone, Debug)]
pub struct AnyHex;
unsafe impl CharSet for AnyHex {
	fn chars(self) -> &'static str {
		LowerHex.chars()
	}
	fn lut(self) -> (char, &'static [u8]) {
		static MAP: [u8; 55] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255, 255, 255, 255, 255, 10, 11, 12, 13, 14, 15, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 10, 11, 12, 13, 14, 15];
		(48 as char, &MAP)
	}
	fn encode_char(self, index: u8) -> Result<char, u8> {
		LowerHex.encode_char(index)
	}
	#[cfg(not(feature = "lut"))]
	fn decode_char(self, chr: char) -> Result<u8, char> {
		match chr {
			'0'...'9' => Ok(chr as u8 - b'0'),
			'a'...'f' => Ok(chr as u8 - b'a' + 10),
			'A'...'F' => Ok(chr as u8 - b'A' + 10),
			_ => Err(chr),
		}
	}
}

//----------------------------------------------------------------

pub struct EncoderT<I, C>(::core::iter::FlatMap<encode::HexEncoder<I, C>, encode::Chars, fn(encode::Chars) -> encode::Chars>);
impl<I, C> Iterator for EncoderT<I, C>
	where I: Iterator<Item = u8>,
	      C: CharSet,
{
	type Item = char;
	fn next(&mut self) -> Option<char> {
		self.0.next()
	}
}
pub struct DecoderT<I, C>(decode::HexDecoder<I, C>);
impl<I, C> Iterator for DecoderT<I, C>
	where I: Iterator<Item = char>,
	      C: CharSet,
{
	type Item = Result<u8, ::Error>;
	fn next(&mut self) -> Option<Result<u8, ::Error>> {
		self.0.next()
	}
}

macro_rules! impl_encoding {
	($charset:ty) => {
		impl ::Encoding for $charset {
			fn alphabet(self) -> &'static str { self.chars() }
			fn encode<B: ::EncodeBuf>(self, bytes: &[u8], buffer: B) -> B::Output { encode(bytes, self, buffer) }
			fn decode<B: ::DecodeBuf>(self, string: &str, buffer: B) -> Result<B::Output, ::Error> { decode(string, self, buffer) }
		}
		impl<I: Iterator<Item = u8>> ::Encoder<I> for $charset {
			type Encoder = EncoderT<I, Self>;
			fn encoder(self, iter: I) -> Self::Encoder {
				EncoderT(encode::HexEncoder::new(iter, self).flat_map(super::id))
			}
		}
		impl<I: Iterator<Item = char>> ::Decoder<I> for $charset {
			type Decoder = DecoderT<I, Self>;
			fn decoder(self, iter: I) -> Self::Decoder {
				DecoderT(decode::HexDecoder::new(iter, self))
			}
		}
	}
}
impl_encoding!(LowerHex);
impl_encoding!(UpperHex);
impl_encoding!(AnyHex);

pub fn encode<C: CharSet, B: ::EncodeBuf>(bytes: &[u8], charset: C, mut buffer: B) -> B::Output {
	// Hex encodes each byte into two chars
	let encode_len = bytes.len() * 2;
	unsafe {
		let ptr = buffer.alloc(encode_len);
		let mut index = 0;
		for chars in encode::HexEncoder::new(bytes.iter().cloned(), charset) {
			*ptr.offset(index as isize) = chars.high() as u8;
			*ptr.offset(index as isize + 1) = chars.low() as u8;
			index += 2;
		}
		debug_assert_eq!(index, encode_len);
		buffer.commit(encode_len)
	}
}

pub fn decode<C: CharSet, B: ::DecodeBuf>(string: &str, charset: C, mut buffer: B) -> Result<B::Output, ::Error> {
	// Hex decodes a byte for every two chars
	let decode_len = string.len() / 2;
	unsafe {
		let ptr = buffer.alloc(decode_len);
		let mut index = 0;
		for item in decode::HexDecoder::new(string.chars(), charset) {
			*ptr.offset(index as isize) = item?;
			index += 1;
		}
		debug_assert_eq!(index, decode_len);
		Ok(buffer.commit(decode_len))
	}
}

//----------------------------------------------------------------

#[cfg(test)]
mod tests;
