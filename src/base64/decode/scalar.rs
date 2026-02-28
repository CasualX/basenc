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

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;
	*dest.add(2) = c << 6 | d;

	Ok(dest.add(3))
}

// aaaaaabb bbbbcccc 00------
unsafe fn decode_3bytes(chunk: &[u8; 3], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;

	if c & 0x3 != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;

	Ok(dest.add(2))
}

// aaaaaabb 0000----
unsafe fn decode_2bytes(chunk: &[u8; 2], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;

	if b & 0xf != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = a << 2 | b >> 4;

	Ok(dest.add(1))
}

pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while let [c0, c1, c2, c3, ref rest @ ..] = *string {
		let chunk = [c0, c1, c2, c3];

		if !matches!(pad, Padding::None) && chunk[3] == PAD_CHAR {
			if chunk[2] == PAD_CHAR {
				dest = decode_2bytes(&[c0, c1], base, dest)?;
			}
			else {
				dest = decode_3bytes(&[c0, c1, c2], base, dest)?;
			}
		}
		else {
			dest = decode_4bytes(&chunk, base, dest)?;
		}

		string = rest;
	}

	if !string.is_empty() {
		if matches!(pad, Padding::Strict) {
			return Err(crate::Error::IncorrectLength);
		}

		// Decode remaining bytes
		dest = match *string {
			[c0, c1, c2] => decode_3bytes(&[c0, c1, c2], base, dest)?,
			[c0, c1] => decode_2bytes(&[c0, c1], base, dest)?,
			_ => return Err(crate::Error::IncorrectLength),
		};
	}

	Ok(dest)
}
