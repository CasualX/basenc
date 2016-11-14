/*!
Base64 codec.
*/

pub mod encode;
pub mod decode;

//----------------------------------------------------------------

/// Base64 charset.
///
/// # Safety
///
/// The characters in the alphabet must be ASCII.
pub unsafe trait CharSet: Copy {
	/// Lookup table for index to char.
	fn chars(self) -> &'static str;
	/// Lookup table for char to index.
	fn lut(self) -> (char, &'static [u8]);
	/// Returns the padding character, if any.
	fn padding(self) -> Option<char>;
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

/// Base64 standard charset.
///
/// Alphabet is `"A-Za-z0-9+/"` with strict `'='` padding.
#[derive(Copy, Clone, Debug)]
pub struct Base64Std;
unsafe impl CharSet for Base64Std {
	fn chars(self) -> &'static str {
		"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
	}
	fn lut(self) -> (char, &'static [u8]) {
		static MAP: [u8; 80] = [62, 255, 255, 255, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 255, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 255, 255, 255, 255, 255, 255, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51];
		(43 as char, &MAP)
	}
	fn padding(self) -> Option<char> {
		Some('=')
	}
	#[cfg(not(feature = "lut"))]
	fn encode_char(self, index: u8) -> Result<char, u8> {
		match index {
			0...25 => Ok((index + b'A') as char),
			26...51 => Ok((index - 26 + b'a') as char),
			52...61 => Ok((index - 52 + b'0') as char),
			62 => Ok('+'),
			63 => Ok('/'),
			_ => Err(index),
		}
		// Holy optimizing batman, this gets vectorized except it isn't useful...
		// if index < 26 { Ok((index + b'A') as char) }
		// else if index < 52 { Ok((index - 26 + b'a') as char) }
		// else if index < 62 { Ok((index - 52 + b'0') as char) }
		// else if index < 63 { Ok('+') }
		// else if index < 64 { Ok('/') }
		// else { Err(index) }
	}
	#[cfg(not(feature = "lut"))]
	fn decode_char(self, chr: char) -> Result<u8, char> {
		match chr {
			'A'...'Z' => Ok(chr as u8 - b'A'),
			'a'...'z' => Ok(chr as u8 - b'a' + 26),
			'0'...'9' => Ok(chr as u8 - b'0' + 52),
			'+' => Ok(62),
			'/' => Ok(63),
			_ => Err(chr),
		}
	}
}

//----------------------------------------------------------------

/// Base64 url-safe charset.
///
/// Alphabet is `"A-Za-z0-9-_"` without padding.
#[derive(Copy, Clone, Debug)]
pub struct Base64Url;
unsafe impl CharSet for Base64Url {
	fn chars(self) -> &'static str {
		"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
	}
	fn lut(self) -> (char, &'static [u8]) {
		static MAP: [u8; 78] = [62, 255, 255, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 255, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 255, 255, 255, 255, 63, 255, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51];
		(45 as char, &MAP)
	}
	fn padding(self) -> Option<char> {
		None
	}
	#[cfg(not(feature = "lut"))]
	fn encode_char(self, index: u8) -> Result<char, u8> {
		match index {
			0...25 => Ok((index + b'A') as char),
			26...51 => Ok((index - 26 + b'a') as char),
			52...61 => Ok((index - 52 + b'0') as char),
			62 => Ok('-'),
			63 => Ok('_'),
			_ => Err(index),
		}
	}
	#[cfg(not(feature = "lut"))]
	fn decode_char(self, chr: char) -> Result<u8, char> {
		match chr {
			'A'...'Z' => Ok(chr as u8 - b'A'),
			'a'...'z' => Ok(chr as u8 - b'a' + 26),
			'0'...'9' => Ok(chr as u8 - b'0' + 52),
			'-' => Ok(62),
			'_' => Ok(63),
			_ => Err(chr),
		}
	}
}

//----------------------------------------------------------------
// Good god these types...

use ::core::iter;

/// Encoder adapter type.
///
/// Adapts an iterator over `u8` into an iterator over `char` with specified charset.
pub struct EncoderT<I, C>(iter::FlatMap<
	encode::ChunkAdapter<encode::ChunkValIter<I>, C>,
	encode::Chars,
	fn(encode::Chars) -> encode::Chars
>);
impl<I, C> Iterator for EncoderT<I, C>
	where I: Iterator<Item = u8>,
	      C: CharSet,
{
	type Item = char;
	fn next(&mut self) -> Option<char> {
		self.0.next()
	}
}
/// Returns the encoder adapter for a given iterator and charset.
pub fn encoder<I: Iterator<Item = u8>, C: CharSet>(iter: I, charset: C) -> EncoderT<I, C> {
	EncoderT(encode::ChunkAdapter::new(
		encode::ChunkValIter::new(iter),
		charset
	).flat_map(super::id))
}

fn result_flat_map(v: Result<decode::Bytes, ::Error>) -> super::ResultFlatMap<decode::BytesIter, ::Error> {
	super::ResultFlatMap::new(v)
}
/// Decoder adapter type.
///
/// Adapts an iterator over `char` into an iterator over `Result<u8, ::Error>` with specified charset.
///
/// Good lord would you look at this type...
pub struct DecoderT<I, C>(iter::FlatMap<
	iter::Map<
		decode::ChunkAdapter<decode::ChunkValIter<I>, C>,
		fn(Result<decode::Bytes, ::Error>) -> super::ResultFlatMap<decode::BytesIter, ::Error>,
	>,
	super::ResultFlatMap<decode::BytesIter, ::Error>,
	fn(super::ResultFlatMap<decode::BytesIter, ::Error>) -> super::ResultFlatMap<decode::BytesIter, ::Error>,
>);
impl<I, C> Iterator for DecoderT<I, C>
	where I: Iterator<Item = char>,
	      C: CharSet,
{
	type Item = Result<u8, ::Error>;
	fn next(&mut self) -> Option<Result<u8, ::Error>> {
		self.0.next()
	}
}
/// Returns the decoder adapter for a given iterator and charset.
pub fn decoder<I: Iterator<Item = char>, C: CharSet>(iter: I, charset: C) -> DecoderT<I, C> {
	DecoderT(decode::ChunkAdapter::new(
		decode::ChunkValIter::new(iter),
		charset
	).map(
		result_flat_map as fn(Result<decode::Bytes, ::Error>) -> super::ResultFlatMap<decode::BytesIter, ::Error>
	).flat_map(super::id))
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
				encoder(iter, self)
			}
		}
		impl<I: Iterator<Item = char>> ::Decoder<I> for $charset {
			type Decoder = DecoderT<I, Self>;
			fn decoder(self, iter: I) -> Self::Decoder {
				decoder(iter, self)
			}
		}
	}
}
impl_encoding!(Base64Std);
impl_encoding!(Base64Url);

#[inline(always)]
fn encode<C: CharSet, B: ::EncodeBuf>(bytes: &[u8], charset: C, mut buffer: B) -> B::Output {
	unsafe {
		// Base64 encodes 3 bytes into 4 characters
		let encode_len = (bytes.len() + 2) / 3 * 4;
		let ptr = buffer.alloc(encode_len);
		#[inline(never)]
		fn lo_op<C: CharSet>(bytes: &[u8], charset: C, ptr: *mut u8) -> usize {
			unsafe {
				let mut index = 0;
				for chr in encode::ChunkAdapter::new(encode::ChunkRefIter::new(bytes), charset).flat_map(super::id) {
					*ptr.offset(index as isize) = chr as u8;
					index += 1;
				}
				index
			}
		};
		let final_len = lo_op(bytes, charset, ptr);
		debug_assert!(final_len <= encode_len);
		buffer.commit(final_len)
	}
}

#[inline(always)]
fn decode<C: CharSet, B: ::DecodeBuf>(string: &str, charset: C, mut buffer: B) -> Result<B::Output, ::Error> {
	unsafe {
		// Base64 decodes 4 characters into 3 bytes
		let decode_len = (string.len() + 3) / 4 * 3;
		let ptr = buffer.alloc(decode_len);
		#[inline(never)]
		fn lo_op<C: CharSet>(string: &str, charset: C, ptr: *mut u8) -> Result<usize, ::Error> {
			unsafe {
				let mut index = 0;
				for item in decoder(string.chars(), charset) {
					*ptr.offset(index as isize) = item?;
					index += 1;
				}
				Ok(index)
			}
		}
		let final_len = lo_op(string, charset, ptr)?;
		debug_assert!(final_len <= decode_len);
		Ok(buffer.commit(final_len))
	}
}

//----------------------------------------------------------------

#[cfg(test)]
mod tests;
