use super::*;

fn lookup(byte: u8, lut: &[u8; 128]) -> Result<u8, crate::Error> {
	if byte as usize >= lut.len() {
		return Err(crate::Error::InvalidCharacter);
	}
	let v = lut[byte as usize];
	if v >= 64 {
		return Err(crate::Error::InvalidCharacter);
	}
	Ok(v)
}

// aaaaaabb bbbbcccc ccdddddd
unsafe fn decode_4bytes(chunk: &[u8; 4], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;

	*dest.add(0) = (a << 2 | b >> 4) as u8;
	*dest.add(1) = (b << 4 | c >> 2) as u8;
	*dest.add(2) = (c << 6 | d) as u8;

	Ok(dest.add(3))
}

// aaaaaabb bbbbcccc cc------
unsafe fn decode_3bytes(chunk: &[u8; 3], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;

	if c & 0x3 != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = (a << 2 | b >> 4) as u8;
	*dest.add(1) = (b << 4 | c >> 2) as u8;

	Ok(dest.add(2))
}

// aaaaaabb bbbb----
unsafe fn decode_2bytes(chunk: &[u8; 2], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;

	if b & 0xf != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = (a << 2 | b >> 4) as u8;

	Ok(dest.add(1))
}

pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 4 {
		let chunk = &*(string.as_ptr() as *const [u8; 4]);

		if !matches!(pad, Padding::None) && chunk[3] == PAD_CHAR {
			if chunk[2] == PAD_CHAR {
				dest = decode_2bytes(&*(chunk as *const _ as *const [u8; 2]), base, dest)?;
			}
			else {
				dest = decode_3bytes(&*(chunk as *const _ as *const [u8; 3]), base, dest)?;
			}
		}
		else {
			dest = decode_4bytes(chunk, base, dest)?;
		}

		string = &string[4..];
	}

	if string.len() != 0 {
		if matches!(pad, Padding::Strict) {
			return Err(crate::Error::IncorrectLength);
		}

		// Decode remaining bytes
		dest = match string.len() {
			3 => decode_3bytes(&*(string.as_ptr() as *const [u8; 3]), base, dest)?,
			2 => decode_2bytes(&*(string.as_ptr() as *const [u8; 2]), base, dest)?,
			_ => return Err(crate::Error::IncorrectLength),
		};
	}

	Ok(dest)
}
