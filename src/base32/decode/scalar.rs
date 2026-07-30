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

// aaaaabbb bbcccccd ddddeeee efffffgg ggghhhhh
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

	*dest.add(0) = a << 3 | b >> 2;
	*dest.add(1) = b << 6 | c << 1 | d >> 4;
	*dest.add(2) = d << 4 | e >> 1;
	*dest.add(3) = e << 7 | f << 2 | g >> 3;
	*dest.add(4) = g << 5 | h;

	Ok(dest.add(5))
}

// aaaaabbb bbcccccd ddddeeee efffffgg 000-----
#[inline]
unsafe fn decode_7bytes(chunk: &[u8; 7], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;
	let e = lookup(chunk[4], &base.lut)?;
	let f = lookup(chunk[5], &base.lut)?;
	let g = lookup(chunk[6], &base.lut)?;

	if g & 0x7 != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = a << 3 | b >> 2;
	*dest.add(1) = b << 6 | c << 1 | d >> 4;
	*dest.add(2) = d << 4 | e >> 1;
	*dest.add(3) = e << 7 | f << 2 | g >> 3;

	Ok(dest.add(4))
}

// aaaaabbb bbcccccd ddddeeee 0------- --------
#[inline]
unsafe fn decode_5bytes(chunk: &[u8; 5], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;
	let e = lookup(chunk[4], &base.lut)?;

	if e & 0x1 != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = a << 3 | b >> 2;
	*dest.add(1) = b << 6 | c << 1 | d >> 4;
	*dest.add(2) = d << 4 | e >> 1;

	Ok(dest.add(3))
}

// aaaaabbb bbcccccd 0000---- -------- --------
#[inline]
unsafe fn decode_4bytes(chunk: &[u8; 4], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;
	let c = lookup(chunk[2], &base.lut)?;
	let d = lookup(chunk[3], &base.lut)?;

	if d & 0xf != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = a << 3 | b >> 2;
	*dest.add(1) = b << 6 | c << 1 | d >> 4;

	Ok(dest.add(2))
}

// aaaaabbb 00------ -------- -------- --------
#[inline]
unsafe fn decode_2bytes(chunk: &[u8; 2], base: &Base32, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(chunk[0], &base.lut)?;
	let b = lookup(chunk[1], &base.lut)?;

	if b & 0x3 != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = a << 3 | b >> 2;

	Ok(dest.add(1))
}

pub unsafe fn decode(mut string: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 8 {
		dest = decode_chunk(&mut string, base, pad, dest)?;
	}

	if !string.is_empty() {
		if matches!(pad, Padding::Required) {
			return Err(crate::Error::IncorrectLength);
		}

		dest = match *string {
			[c0, c1, c2, c3, c4, c5, c6] => decode_7bytes(&[c0, c1, c2, c3, c4, c5, c6], base, dest)?,
			[c0, c1, c2, c3, c4] => decode_5bytes(&[c0, c1, c2, c3, c4], base, dest)?,
			[c0, c1, c2, c3] => decode_4bytes(&[c0, c1, c2, c3], base, dest)?,
			[c0, c1] => decode_2bytes(&[c0, c1], base, dest)?,
			_ => return Err(crate::Error::IncorrectLength),
		};
	}

	Ok(dest)
}

#[inline]
pub unsafe fn decode_chunk(string: &mut &[u8], base: &Base32, pad: Padding, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let [c0, c1, c2, c3, c4, c5, c6, c7, ref rest @ ..] = **string
	else {
		return Err(crate::Error::IncorrectLength);
	};
	let chunk = [c0, c1, c2, c3, c4, c5, c6, c7];

	let dest = if !matches!(pad, Padding::Forbidden) && chunk[7] == PAD_CHAR {
		if !matches!(pad, Padding::Internal) && rest.iter().any(|&byte| byte != PAD_CHAR) {
			return Err(crate::Error::InvalidCharacter);
		}
		if chunk[6] == PAD_CHAR && chunk[5] == PAD_CHAR {
			if chunk[4] == PAD_CHAR {
				if chunk[3] == PAD_CHAR && chunk[2] == PAD_CHAR {
					decode_2bytes(&[c0, c1], base, dest)?
				}
				else {
					decode_4bytes(&[c0, c1, c2, c3], base, dest)?
				}
			}
			else {
				decode_5bytes(&[c0, c1, c2, c3, c4], base, dest)?
			}
		}
		else {
			decode_7bytes(&[c0, c1, c2, c3, c4, c5, c6], base, dest)?
		}
	}
	else {
		decode_8bytes(&chunk, base, dest)?
	};

	*string = rest;
	Ok(dest)
}
