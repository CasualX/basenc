use super::*;

#[inline]
fn lookup(byte: u8, offset: usize) -> Result<u8, crate::Error> {
	if byte as usize >= LUT.len() {
		return Err(crate::Error::new(crate::ErrorKind::InvalidCharacter, offset));
	}
	let value = LUT[byte as usize];
	if value >= 64 {
		return Err(crate::Error::new(crate::ErrorKind::InvalidCharacter, offset));
	}
	Ok(value)
}

#[inline]
unsafe fn decode_4bytes([c0, c1, c2, c3]: &[u8; 4], dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(*c0, 0)?;
	let b = lookup(*c1, 1)?;
	let c = lookup(*c2, 2)?;
	let d = lookup(*c3, 3)?;

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;
	*dest.add(2) = c << 6 | d;
	Ok(dest.add(3))
}

#[inline]
unsafe fn decode_3bytes([c0, c1, c2]: &[u8; 3], dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(*c0, 0)?;
	let b = lookup(*c1, 1)?;
	let c = lookup(*c2, 2)?;
	if c & 0x3 != 0 {
		return Err(crate::Error::new(crate::ErrorKind::NonCanonical, 2));
	}

	*dest.add(0) = a << 2 | b >> 4;
	*dest.add(1) = b << 4 | c >> 2;
	Ok(dest.add(2))
}

#[inline]
unsafe fn decode_2bytes([c0, c1]: &[u8; 2], dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a = lookup(*c0, 0)?;
	let b = lookup(*c1, 1)?;
	if b & 0xf != 0 {
		return Err(crate::Error::new(crate::ErrorKind::NonCanonical, 1));
	}

	*dest = a << 2 | b >> 4;
	Ok(dest.add(1))
}

pub unsafe fn decode(mut string: &[u8], pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 4 {
		let offset = input_len - string.len();
		dest = decode_chunk(&mut string, pad, dest).map_err(|error| error.shifted(offset))?;
	}

	if !string.is_empty() {
		if pad.decode_requires_padding() {
			return Err(crate::Error::new(crate::ErrorKind::IncorrectLength, input_len));
		}
		let offset = input_len - string.len();
		dest = match *string {
			[c0, c1, c2] => decode_3bytes(&[c0, c1, c2], dest).map_err(|error| error.shifted(offset))?,
			[c0, c1] => decode_2bytes(&[c0, c1], dest).map_err(|error| error.shifted(offset))?,
			_ => return Err(crate::Error::new(crate::ErrorKind::IncorrectLength, input_len)),
		};
	}

	Ok(dest)
}

#[inline]
pub unsafe fn decode_chunk(string: &mut &[u8], pad: Padding, dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let [c0, c1, c2, c3, ref rest @ ..] = **string
	else {
		return Err(crate::Error::new(crate::ErrorKind::IncorrectLength, string.len()));
	};

	let dest = if pad.decode_allows_padding() && c3 == PAD_CHAR {
		if !pad.decode_allows_internal_padding()
			&& let Some(offset) = rest.iter().position(|&byte| byte != PAD_CHAR)
		{
			return Err(crate::Error::new(crate::ErrorKind::InvalidCharacter, 4 + offset));
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
