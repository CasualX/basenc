use super::*;

#[inline]
fn lookup(byte: u8) -> Result<u8, crate::Error> {
	if byte as usize >= LUT.len() {
		return Err(crate::Error::InvalidCharacter);
	}
	let value = LUT[byte as usize];
	if value >= 64 {
		return Err(crate::Error::InvalidCharacter);
	}
	Ok(value)
}

#[inline]
unsafe fn decode_4bytes([c0, c1, c2, c3]: &[u8; 4], dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(*c0)?;
	let b = lookup(*c1)?;
	let c = lookup(*c2)?;
	let d = lookup(*c3)?;

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;
	*dest.add(2) = c << 6 | d;
	Ok(dest.add(3))
}

#[inline]
unsafe fn decode_3bytes([c0, c1, c2]: &[u8; 3], dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(*c0)?;
	let b = lookup(*c1)?;
	let c = lookup(*c2)?;
	if c & 0x3 != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;
	Ok(dest.add(2))
}

#[inline]
unsafe fn decode_2bytes([c0, c1]: &[u8; 2], dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(*c0)?;
	let b = lookup(*c1)?;
	if b & 0xf != 0 {
		return Err(crate::Error::NonCanonical);
	}

	*dest = a << 2 | b >> 4;
	Ok(dest.add(1))
}

pub unsafe fn decode(mut string: &[u8], pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 4 {
		dest = decode_chunk(&mut string, pad, dest)?;
	}

	if !string.is_empty() {
		if matches!(pad, Padding::Required) {
			return Err(crate::Error::IncorrectLength);
		}
		dest = match *string {
			[c0, c1, c2] => decode_3bytes(&[c0, c1, c2], dest)?,
			[c0, c1] => decode_2bytes(&[c0, c1], dest)?,
			_ => return Err(crate::Error::IncorrectLength),
		};
	}

	Ok(dest)
}

#[inline]
pub unsafe fn decode_chunk(string: &mut &[u8], pad: Padding, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let [c0, c1, c2, c3, ref rest @ ..] = **string
	else {
		return Err(crate::Error::IncorrectLength);
	};

	let dest = if !matches!(pad, Padding::Forbidden) && c3 == PAD_CHAR {
		if !matches!(pad, Padding::Internal) && rest.iter().any(|&byte| byte != PAD_CHAR) {
			return Err(crate::Error::InvalidCharacter);
		}
		if c2 == PAD_CHAR {
			decode_2bytes(&[c0, c1], dest)?
		}
		else {
			decode_3bytes(&[c0, c1, c2], dest)?
		}
	}
	else {
		decode_4bytes(&[c0, c1, c2, c3], dest)?
	};

	*string = rest;
	Ok(dest)
}
