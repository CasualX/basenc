/*!
Gory encoder details.
*/

use super::CharSet;

/// For every byte, two hexadecimal digits are produced.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Chars(char, char);
impl Chars {
	pub fn new(high: char, low: char) -> Chars {
		Chars(high, low)
	}
	pub fn high(self) -> char {
		self.0
	}
	pub fn low(self) -> char {
		self.1
	}
}
impl IntoIterator for Chars {
	type Item = char;
	type IntoIter = CharsIter;
	fn into_iter(self) -> CharsIter {
		CharsIter {
			index: 0,
			unit: self,
		}
	}
}

/// Iterate over the component `char`s.
///
/// Allows `flat_map`ing an iterator over `Chars`s.
#[derive(Copy, Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CharsIter {
	index: usize,
	unit: Chars,
}
impl Iterator for CharsIter {
	type Item = char;
	#[inline(always)]
	fn next(&mut self) -> Option<char> {
		// Very naive implementation, should allow loop-unroll optimization
		match self.index {
			0 => {
				self.index = 1;
				Some(self.unit.high())
			},
			1 => {
				self.index = 2;
				Some(self.unit.low())
			},
			_ => None,
		}
	}
	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(2))
	}
}
impl ExactSizeIterator for CharsIter {}

//----------------------------------------------------------------

/// Encoder taking `u8`s and producing `Chars`'.
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HexEncoder<I, C> {
	iter: I,
	charset: C,
}
impl<I: Iterator<Item = u8>, C: CharSet> HexEncoder<I, C> {
	pub fn new(iter: I, charset: C) -> HexEncoder<I, C> {
		HexEncoder {
			iter: iter,
			charset: charset,
		}
	}
}
impl<I: Iterator<Item = u8>, C: CharSet> Iterator for HexEncoder<I, C> {
	type Item = Chars;
	fn next(&mut self) -> Option<Chars> {
		self.iter.next().map(|byte| {
			// High nibble to char
			let high = {
				let nibble = (byte >> 4) & 0x0F;
				self.charset.encode_char(nibble).unwrap()
			};
			// Low nibble to char
			let low = {
				let nibble = byte & 0x0F;
				self.charset.encode_char(nibble).unwrap()
			};
			Chars::new(high, low)
		})
	}
}

//----------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;
	use super::super::*;
	use ::std::iter::once;

	#[test]
	fn hex_encoder() {
		assert_eq!(
			HexEncoder::new(once(0x5A), LowerHex).next(),
			Some(Chars('5', 'a'))
		);
		assert_eq!(
			HexEncoder::new(once(0xFE), UpperHex).next(),
			Some(Chars('F', 'E'))
		);
	}
}
