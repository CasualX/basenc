use super::*;

fn lookup(byte: u8, lut: &[u8; 127]) -> Result<u8, crate::Error> {
	if byte >= lut.len() as u8 {
		return Err(crate::Error::InvalidChar);
	}
	let v = lut[byte as usize];
	if v >= 64 {
		return Err(crate::Error::InvalidChar);
	}
	Ok(v)
}

// aaaaaabb bbbbcccc ccdddddd
unsafe fn decode_4bytes(bytes: &[u8; 4], base: &Base64, ptr: *mut u8) -> Result<(), crate::Error> {
	let a = lookup(bytes[0], &base.lut)?;
	let b = lookup(bytes[1], &base.lut)?;
	let c = lookup(bytes[2], &base.lut)?;
	let d = lookup(bytes[3], &base.lut)?;

	*ptr.add(0) = (a << 2 | b >> 4) as u8;
	*ptr.add(1) = (b << 4 | c >> 2) as u8;
	*ptr.add(2) = (c << 6 | d) as u8;

	Ok(())
}

// aaaaaabb bbbbcccc cc------
unsafe fn decode_3bytes(bytes: &[u8; 3], base: &Base64, ptr: *mut u8) -> Result<(), crate::Error> {
	let a = lookup(bytes[0], &base.lut)?;
	let b = lookup(bytes[1], &base.lut)?;
	let c = lookup(bytes[2], &base.lut)?;

	if c & 0x3 != 0 {
		return Err(crate::Error::Denormal);
	}

	*ptr.add(0) = (a << 2 | b >> 4) as u8;
	*ptr.add(1) = (b << 4 | c >> 2) as u8;

	Ok(())
}

// aaaaaabb bbbb----
unsafe fn decode_2bytes(bytes: &[u8; 2], base: &Base64, ptr: *mut u8) -> Result<(), crate::Error> {
	let a = lookup(bytes[0], &base.lut)?;
	let b = lookup(bytes[1], &base.lut)?;

	if b & 0xf != 0 {
		return Err(crate::Error::Denormal);
	}

	*ptr.add(0) = (a << 2 | b >> 4) as u8;

	Ok(())
}

pub unsafe fn decode(mut string: &[u8], base: &Base64, mut ptr: *mut u8) -> Result<(), crate::Error> {
	if string.len() == 0 {
		return Ok(());
	}
	
	while string.len() > 4 {
		let bytes = &*(string.as_ptr() as *const [u8; 4]);
		decode_4bytes(bytes, base, ptr)?;
		ptr = ptr.add(3);
		string = &string[4..];
	}

	match string.len() {
		4 => {
			if matches!(base.padding, Padding::Optional | Padding::Strict) {
				if string[2] == PAD_CHAR && string[3] == PAD_CHAR {
					decode_2bytes(&*(string.as_ptr() as *const [u8; 2]), base, ptr)?;
				}
				else if string[3] == PAD_CHAR {
					decode_3bytes(&*(string.as_ptr() as *const [u8; 3]), base, ptr)?;
				}
				else {
					decode_4bytes(&*(string.as_ptr() as *const [u8; 4]), base, ptr)?;
				}
			}
			else {
				decode_4bytes(&*(string.as_ptr() as *const [u8; 4]), base, ptr)?;
			}
		}
		3 => {
			if matches!(base.padding, Padding::Strict) {
				return Err(crate::Error::BadLength);
			}
			decode_3bytes(&*(string.as_ptr() as *const [u8; 3]), base, ptr)?;
		}
		2 => {
			if matches!(base.padding, Padding::Strict) {
				return Err(crate::Error::BadLength);
			}
			decode_2bytes(&*(string.as_ptr() as *const [u8; 2]), base, ptr)?;
		}
		_ => {
			return Err(crate::Error::BadLength);
		}
	}

	Ok(())
}
