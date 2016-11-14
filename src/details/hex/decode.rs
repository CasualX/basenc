/*!
Gory decoder details.
*/

use super::CharSet;

//----------------------------------------------------------------

/// Decoder taking `char`s and producing `u8`s.
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct HexDecoder<I, C> {
	iter: I,
	charset: C,
}
impl<I: Iterator<Item = char>, C: CharSet> HexDecoder<I, C> {
	pub fn new(iter: I, charset: C) -> HexDecoder<I, C> {
		HexDecoder {
			iter: iter,
			charset: charset,
		}
	}
}
impl<I: Iterator<Item = char>, C: CharSet> Iterator for HexDecoder<I, C> {
	type Item = Result<u8, ::Error>;
	#[inline(always)]
	fn next(&mut self) -> Option<Result<u8, ::Error>> {
		let mut byte_buf = 0;
		let mut done = true;
		loop {
			// Get next char
			let c = if let Some(c) = self.iter.next() { c }
			// If `None` on first iteration, there are no more digits
			else if done { return None; }
			// Else there is an unpaired hex digit
			else { return Some(Err(::Error::BadLength)) };
			// Get the index for this char
			let nibble = match self.charset.decode_char(c) {
				Ok(index) => index,
				Err(chr) => return Some(Err(::Error::InvalidChar(chr))),
			};
			byte_buf = (byte_buf << 4) | nibble;
			// End of iteration
			done = !done;
			if done { break; }
		}
		Some(Ok(byte_buf))
	}
}
