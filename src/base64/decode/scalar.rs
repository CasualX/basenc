use super::*;

fn lookup(byte: u8, lut: &[u8; 128], offset: usize) -> Result<u8, crate::Error> {
	if byte as usize >= lut.len() {
		return Err(crate::Error::new(crate::ErrorKind::InvalidCharacter, offset));
	}
	let v = lut[byte as usize];
	if v >= 64 {
		return Err(crate::Error::new(crate::ErrorKind::InvalidCharacter, offset));
	}
	Ok(v)
}

// aaaaaabb bbbbcccc ccdddddd
unsafe fn decode_4bytes(chunk: &[u8; 4], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut, 0)?;
	let b = lookup(chunk[1], &base.lut, 1)?;
	let c = lookup(chunk[2], &base.lut, 2)?;
	let d = lookup(chunk[3], &base.lut, 3)?;

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;
	*dest.add(2) = c << 6 | d;

	Ok(dest.add(3))
}

// aaaaaabb bbbbcccc 00------
unsafe fn decode_3bytes(chunk: &[u8; 3], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut, 0)?;
	let b = lookup(chunk[1], &base.lut, 1)?;
	let c = lookup(chunk[2], &base.lut, 2)?;

	if c & 0x3 != 0 {
		return Err(crate::Error::new(crate::ErrorKind::NonCanonical, 2));
	}

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;

	Ok(dest.add(2))
}

// aaaaaabb 0000----
unsafe fn decode_2bytes(chunk: &[u8; 2], base: &Base64, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut, 0)?;
	let b = lookup(chunk[1], &base.lut, 1)?;

	if b & 0xf != 0 {
		return Err(crate::Error::new(crate::ErrorKind::NonCanonical, 1));
	}

	*dest.add(0) = a << 2 | b >> 4;

	Ok(dest.add(1))
}

pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 4 {
		let offset = input_len - string.len();
		dest = decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
	}

	if !string.is_empty() {
		if matches!(pad, Padding::Required) {
			return Err(crate::Error::new(crate::ErrorKind::IncorrectLength, input_len));
		}

		// Decode remaining bytes
		let offset = input_len - string.len();
		dest = match *string {
			[c0, c1, c2] => decode_3bytes(&[c0, c1, c2], base, dest).map_err(|error| error.shifted(offset))?,
			[c0, c1] => decode_2bytes(&[c0, c1], base, dest).map_err(|error| error.shifted(offset))?,
			_ => return Err(crate::Error::new(crate::ErrorKind::IncorrectLength, input_len)),
		};
	}

	Ok(dest)
}

#[inline]
pub unsafe fn decode_chunk(string: &mut &[u8], base: &Base64, pad: Padding, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let [c0, c1, c2, c3, ref rest @ ..] = **string
	else {
		return Err(crate::Error::new(crate::ErrorKind::IncorrectLength, string.len()));
	};
	let chunk = [c0, c1, c2, c3];

	let dest = if !matches!(pad, Padding::Forbidden) && chunk[3] == PAD_CHAR {
		if !matches!(pad, Padding::Internal)
			&& let Some(offset) = rest.iter().position(|&byte| byte != PAD_CHAR)
		{
			return Err(crate::Error::new(crate::ErrorKind::InvalidCharacter, 4 + offset));
		}
		if chunk[2] == PAD_CHAR {
			decode_2bytes(&[c0, c1], base, dest)?
		}
		else {
			decode_3bytes(&[c0, c1, c2], base, dest)?
		}
	}
	else {
		decode_4bytes(&chunk, base, dest)?
	};

	*string = rest;
	Ok(dest)
}
