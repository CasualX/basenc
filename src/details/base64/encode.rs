/*!
Gory encoder details.

The encoding is pipelined:

The first stage creates an iterator over `Chunk`s.
This can be created from arbitrary `Iterator<Item = u8>` using `ChunkValIter` or from a `&[u8]` using `ChunkRefIter`.

The second stage tranforms chunks into groups of output characters with the `ChunkAdapter`.

The last stage extracts the individual characters from the adapter.
*/

use super::CharSet;
use super::super::Chunk;

use ::core::ops;

//----------------------------------------------------------------
// Iterate over chunks by value

#[derive(Clone, Debug)]
pub struct ChunkVal {
	len: u8,
	val: [u8; 3],
}
impl Chunk for ChunkVal {
	fn len(&self) -> usize {
		self.len as usize
	}
}
impl ops::Index<usize> for ChunkVal {
	type Output = u8;
	fn index(&self, index: usize) -> &u8 {
		&self.val[index]
	}
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct ChunkValIter<I>(I);
impl<I: Iterator<Item = u8>> ChunkValIter<I> {
	pub fn new(iter: I) -> ChunkValIter<I> {
		ChunkValIter(iter)
	}
}
impl<I: Iterator<Item = u8>> Iterator for ChunkValIter<I> {
	type Item = ChunkVal;
	fn next(&mut self) -> Option<ChunkVal> {
		let mut chunk = ChunkVal { val: [0u8; 3], len: 0 };
		for i in 0..3 {
			if let Some(byte) = self.0.next() {
				chunk.len = i + 1;
				chunk.val[i as usize] = byte;
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
// Iterate over chunks by reference

#[derive(Clone, Debug)]
pub struct ChunkRef<'a>(&'a [u8]);
impl<'a> Chunk for ChunkRef<'a> {
	fn len(&self) -> usize {
		self.0.len()
	}
}
impl<'a> ops::Index<usize> for ChunkRef<'a> {
	type Output = u8;
	fn index(&self, index: usize) -> &u8 {
		self.0.index(index)
	}
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct ChunkRefIter<'a>(&'a [u8]);
impl<'a> ChunkRefIter<'a> {
	pub fn new(slice: &'a [u8]) -> ChunkRefIter<'a> {
		ChunkRefIter(slice)
	}
}
impl<'a> Iterator for ChunkRefIter<'a> {
	type Item = ChunkRef<'a>;
	fn next(&mut self) -> Option<ChunkRef<'a>> {
		if self.0.is_empty() {
			None
		}
		else {
			let len = ::core::cmp::min(self.0.len(), 3);
			let chunk = &self.0[..len];
			self.0 = &self.0[len..];
			Some(ChunkRef(chunk))
		}
	}
}

//----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Chars {
	chars: [char; 4],
	len: u32,
}
impl Chars {
	pub fn new(pad: char, len: usize) -> Chars {
		Chars {
			chars: [pad; 4],
			len: len as u32,
		}
	}
	unsafe fn get(&self, index: usize) -> Option<char> {
		if index >= self.len as usize {
			None
		}
		else {
			Some(*self.chars[..].get_unchecked(index))
		}
	}
}
impl IntoIterator for Chars {
	type Item = char;
	type IntoIter = CharsIter;
	fn into_iter(self) -> CharsIter {
		CharsIter {
			chars: self,
			cur: 0,
		}
	}
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct CharsIter {
	chars: Chars,
	cur: u32,
}
impl Iterator for CharsIter {
	type Item = char;
	fn next(&mut self) -> Option<char> {
		// Help the optimizer out a little :)
		unsafe { self.chars.get(self.cur as usize) }.map(|chr| { self.cur += 1; chr })
	}
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct ChunkAdapter<I, C> {
	iter: I,
	charset: C,
}
impl<T: Chunk<Output = u8>, I: Iterator<Item = T>, C: CharSet> ChunkAdapter<I, C> {
	pub fn new(iter: I, charset: C) -> ChunkAdapter<I, C> {
		ChunkAdapter {
			iter: iter,
			charset: charset,
		}
	}
}
impl<T: Chunk<Output = u8>, I: Iterator<Item = T>, C: CharSet> Iterator for ChunkAdapter<I, C> {
	type Item = Chars;
	fn next(&mut self) -> Option<Chars> {
		self.iter.next().map(|chunk| {
			// Temp space for the 6-bit unsigned ints
			let mut buf = [0u8; 4];
			// Splice the pieces: aaaaaabb bbbbcccc ccdddddd
			if chunk.len() >= 1 {
				buf[0] = chunk[0] >> 2;
				buf[1] = chunk[0] << 4;
				if chunk.len() >= 2 {
					buf[1] |= chunk[1] >> 4;
					buf[2] = chunk[1] << 2;
					if chunk.len() >= 3 {
						buf[2] |= chunk[2] >> 6;
						buf[3] = chunk[2];
					}
				}
			}
			// Optimizer should know this...
			let len = ::core::cmp::min(chunk.len() + 1, 4);
			let mut chars = if let Some(pad) = self.charset.padding() {
				Chars::new(pad, 4)
			}
			else {
				Chars::new('\0', len)
			};
			for i in 0..len {
				chars.chars[i] = self.charset.encode_char(buf[i] & 0x3F).unwrap();
			}
			chars
		})
	}
}
