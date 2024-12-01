
fn decode_nibble(byte: u8) -> Result<u8, crate::Error> {
	match byte {
		b'0'..=b'9' => Ok(byte - b'0'),
		b'a'..=b'f' => Ok(byte - b'a' + 10),
		b'A'..=b'F' => Ok(byte - b'A' + 10),
		_ => Err(crate::Error::InvalidChar),
	}
}

pub unsafe fn decode(mut src: *const u8, mut len: usize, mut dest: *mut u8) -> Result<(), crate::Error> {

	while len > 0 {
		let hi = decode_nibble(*src)?;
		src = src.add(1);
		let lo = decode_nibble(*src)?;
		src = src.add(1);
		*dest = hi << 4 | lo;
		dest = dest.add(1);
		len -= 1;
	}

	Ok(())
}
