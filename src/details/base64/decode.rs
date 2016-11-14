/*!
Gory decoder details.
*/

use super::CharSet;
use super::super::Chunk;

use ::core::ops;

//----------------------------------------------------------------
// Iterate over chunks by value

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkVal {
	chars: [char; 4],
	len: u32,
}
impl Chunk for ChunkVal {
	fn len(&self) -> usize {
		self.len as usize
	}
}
impl ops::Index<usize> for ChunkVal {
	type Output = char;
	fn index(&self, index: usize) -> &char {
		&self.chars[index]
	}
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct ChunkValIter<I>(I);
impl<I: Iterator<Item = char>> ChunkValIter<I> {
	pub fn new(iter: I) -> ChunkValIter<I> {
		ChunkValIter(iter)
	}
}
impl<I: Iterator<Item = char>> Iterator for ChunkValIter<I> {
	type Item = ChunkVal;
	fn next(&mut self) -> Option<ChunkVal> {
		let mut chunk = ChunkVal { chars: ['\0'; 4], len: 0 };
		for i in 0..4 {
			if let Some(chr) = self.0.next() {
				chunk.len = i + 1;
				chunk.chars[i as usize] = chr;
			}
			else if chunk.len == 0 {
				return None;
			}
			else {
				break;
			}
		}
		Some(chunk)
	}
}

//----------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bytes {
	len: u8,
	bytes: [u8; 3],
}
impl Bytes {
	unsafe fn get(&self, index: usize) -> Option<u8> {
		if index >= self.len as usize {
			None
		}
		else {
			Some(*self.bytes[..].get_unchecked(index))
		}
	}
}
impl IntoIterator for Bytes {
	type Item = u8;
	type IntoIter = BytesIter;
	fn into_iter(self) -> BytesIter {
		BytesIter {
			bytes: self,
			cur: 0,
		}
	}
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BytesIter {
	bytes: Bytes,
	cur: u32,
}
impl Iterator for BytesIter {
	type Item = u8;
	fn next(&mut self) -> Option<u8> {
		unsafe { self.bytes.get(self.cur as usize) }.map(|byte| { self.cur += 1; byte })
	}
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct ChunkAdapter<I, C> {
	iter: I,
	charset: C,
}
impl<T: Chunk<Output = char>, I: Iterator<Item = T>, C: CharSet> ChunkAdapter<I, C> {
	pub fn new(iter: I, charset: C) -> ChunkAdapter<I, C> {
		ChunkAdapter {
			iter: iter,
			charset: charset,
		}
	}
}
impl<T: Chunk<Output = char>, I: Iterator<Item = T>, C: CharSet> Iterator for ChunkAdapter<I, C> {
	type Item = Result<Bytes, ::Error>;
	fn next(&mut self) -> Option<Result<Bytes, ::Error>> {
		self.iter.next().map(|chunk| {
			// With padding enforce chunk size
			if self.charset.padding().is_some() && chunk.len() != 4 {
				return Err(::Error::BadLength);
			}
			// De-splice the pieces
			let mut buf: u32 = 0;
			let mut len = 0;
			for i in 0..chunk.len() {
				match self.charset.decode_char(chunk[i]) {
					Ok(byte) => {
						// aaaaaabb bbbbcccc ccdddddd
						buf = (buf << 6) | (byte as u32);
						len = i + 1;
					},
					Err(chr) => {
						if let Some(pad) = self.charset.padding() {
							if pad == chr {
								if i == 2 && chunk[3] != pad {
									return Err(::Error::InvalidChar(chunk[3]));
								}
								break;
							}
							else {
								return Err(::Error::InvalidChar(chr));
							}
						}
						else {
							return Err(::Error::InvalidChar(chr));
						}
					},
				}
			}
			// Length and denormal check
			if len != 4 {
				if len == 3 {
					if (buf & 0x03) != 0 {
						return Err(::Error::Denormal);
					}
					buf = buf << 6;
				}
				else if len == 2 {
					if (buf & 0x0F) != 0 {
						return Err(::Error::Denormal);
					}
					buf = buf << 12;
				}
				else {
					return Err(::Error::BadLength);
				}
			}
			// Compose the bytes
			Ok(Bytes { len: (len - 1) as u8, bytes: [(buf >> 16 & 0xFF) as u8, (buf >> 8 & 0xFF) as u8, (buf & 0xFF) as u8]})
		})
	}
}

//----------------------------------------------------------------

#[cfg(test)]
mod tests {
	use ::std::prelude::v1::*;
	use ::std::iter::once;
	use super::*;
	use super::super::*;

	#[test]
	fn chunk_val_iter() {
		let result = &[
			ChunkVal { chars: ['a', 'b', 'c', 'd'], len: 4 },
			ChunkVal { chars: ['1', '2', '\0', '\0'], len: 2 },
		][..];
		for (left, right) in ChunkValIter::new("abcd12".chars()).zip(result) {
			assert_eq!(&left, right);
		}
	}
	#[test]
	fn bytes_iter() {
		let result = &[
			Bytes { len: 3, bytes: [b'a', b'b', b'c'] },
			Bytes { len: 2, bytes: [b'1', b'2', 0] },
		][..];
		assert_eq!(result.iter().cloned().flat_map(|bytes| bytes).collect::<Vec<_>>(), b"abc12");
	}
	#[test]
	fn chunk_adapter() {
		assert_eq!(
			ChunkAdapter::new(once(ChunkVal { chars: ['c', '3', 'V', 'y'], len: 4 }), Base64Std).next(),
			Some(Ok(Bytes { len: 3, bytes: [b's', b'u', b'r'] }))
		);

		assert_eq!(
			ChunkAdapter::new(once(ChunkVal { chars: ['Z', 'S', '4', '='], len: 4 }), Base64Std).next(),
			Some(Ok(Bytes { len: 2, bytes: [b'e', b'.', 0] }))
		);
		
		assert_eq!(
			ChunkAdapter::new(once(ChunkVal { chars: ['W', 'A', '=', '='], len: 4 }), Base64Std).next(),
			Some(Ok(Bytes { len: 1, bytes: [b'X', 0, 0] }))
		);
	}
}
