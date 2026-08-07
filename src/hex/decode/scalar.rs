
fn decode_nibble(byte: u8, offset: usize) -> Result<u8, crate::Error> {
	match byte {
		b'0'..=b'9' => Ok(byte - b'0'),
		b'a'..=b'f' => Ok(byte - b'a' + 10),
		b'A'..=b'F' => Ok(byte - b'A' + 10),
		_ => Err(crate::Error::new(crate::ErrorKind::InvalidCharacter, offset)),
	}
}

pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while let &[hi, lo, ref rest @ ..] = string {
		let offset = input_len - string.len();
		*dest = decode_nibble(hi, offset)? << 4 | decode_nibble(lo, offset + 1)?;
		dest = dest.add(1);
		string = rest;
	}

	if !string.is_empty() {
		return Err(crate::Error::new(crate::ErrorKind::IncorrectLength, input_len));
	}

	Ok(dest)
}
