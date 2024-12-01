
fn decode_nibble(byte: u8) -> Result<u8, crate::Error> {
	match byte {
		b'0'..=b'9' => Ok(byte - b'0'),
		b'a'..=b'f' => Ok(byte - b'a' + 10),
		b'A'..=b'F' => Ok(byte - b'A' + 10),
		_ => Err(crate::Error::InvalidCharacter),
	}
}

pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while let &[hi, lo, ref rest @ ..] = string {
		*dest = decode_nibble(hi)? << 4 | decode_nibble(lo)?;
		dest = dest.add(1);
		string = rest;
	}

	if string.len() != 0 {
		return Err(crate::Error::IncorrectLength);
	}

	Ok(dest)
}
