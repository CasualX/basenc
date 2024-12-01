use super::*;

#[inline]
fn lookup(byte: u8, lut: &[u8; 128]) -> Result<u8, crate::Error> {
	if byte as usize >= lut.len() {
		return Err(crate::Error::InvalidCharacter);
	}
	let v = lut[byte as usize];
	if v >= 32 {
		return Err(crate::Error::InvalidCharacter);
	}
	Ok(v)
}

#[inline]
unsafe fn decode_8bytes(chunk: &[u8; 8], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;
	let e = lookup(chunk[4], &base.lut)?;
	let f = lookup(chunk[5], &base.lut)?;
	let g = lookup(chunk[6], &base.lut)?;
	let h = lookup(chunk[7], &base.lut)?;

	*dest.add(0) = (a << 3 | b >> 2) as u8;
	*dest.add(1) = (b << 6 | c << 1 | d >> 4) as u8;
	*dest.add(2) = (d << 4 | e >> 1) as u8;
	*dest.add(3) = (e << 7 | f << 2 | g >> 3) as u8;
	*dest.add(4) = (g << 5 | h) as u8;

	Ok(dest.add(5))
}

#[inline]
unsafe fn decode_7bytes(chunk: &[u8; 7], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;
	let e = lookup(chunk[4], &base.lut)?;
	let f = lookup(chunk[5], &base.lut)?;
	let g = lookup(chunk[6], &base.lut)?;

	*dest.add(0) = (a << 3 | b >> 2) as u8;
	*dest.add(1) = (b << 6 | c << 1 | d >> 4) as u8;
	*dest.add(2) = (d << 4 | e >> 1) as u8;
	*dest.add(3) = (e << 7 | f << 2 | g >> 3) as u8;

	Ok(dest.add(4))
}

#[inline]
unsafe fn decode_5bytes(chunk: &[u8; 5], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;
	let e = lookup(chunk[4], &base.lut)?;

	*dest.add(0) = (a << 3 | b >> 2) as u8;
	*dest.add(1) = (b << 6 | c << 1 | d >> 4) as u8;
	*dest.add(2) = (d << 4 | e >> 1) as u8;

	Ok(dest.add(3))
}

#[inline]
unsafe fn decode_4bytes(chunk: &[u8; 4], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;

	*dest.add(0) = (a << 3 | b >> 2) as u8;
	*dest.add(1) = (b << 6 | c << 1 | d >> 4) as u8;

	Ok(dest.add(2))
}

#[inline]
unsafe fn decode_2bytes(chunk: &[u8; 2], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;

	*dest.add(0) = (a << 3 | b >> 2) as u8;

	Ok(dest.add(1))
}

pub unsafe fn decode(mut string: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 8 {
		let chunk = &*(string.as_ptr() as *const [u8; 8]);

		if !matches!(pad, Padding::None) && chunk[7] == PAD_CHAR {
			if chunk[6] == PAD_CHAR && chunk[5] == PAD_CHAR {
				if chunk[4] == PAD_CHAR {
					if chunk[3] == PAD_CHAR && chunk[2] == PAD_CHAR {
						dest = decode_2bytes(&*(chunk as *const _ as *const [u8; 2]), base, dest)?;
					}
					else {
						dest = decode_4bytes(&*(chunk as *const _ as *const [u8; 4]), base, dest)?;
					}
				}
				else {
					dest = decode_5bytes(&*(chunk as *const _ as *const [u8; 5]), base, dest)?;
				}
			}
			else {
				dest = decode_7bytes(&*(chunk as *const _ as *const [u8; 7]), base, dest)?;
			}
		}
		else {
			dest = decode_8bytes(chunk, base, dest)?;
		}

		string = &string[8..];
	}

	if string.len() != 0 {
		if matches!(pad, Padding::Strict) {
			return Err(crate::Error::IncorrectLength);
		}

		dest = match string.len() {
			7 => decode_7bytes(&*(string.as_ptr() as *const [u8; 7]), base, dest)?,
			5 => decode_5bytes(&*(string.as_ptr() as *const [u8; 5]), base, dest)?,
			4 => decode_4bytes(&*(string.as_ptr() as *const [u8; 4]), base, dest)?,
			2 => decode_2bytes(&*(string.as_ptr() as *const [u8; 2]), base, dest)?,
			_ => return Err(crate::Error::IncorrectLength),
		};
	}

	Ok(dest)
}
