use super::*;

// aaaaaabb bbbbcccc ccdddddd --------
#[inline]
unsafe fn encode_word(word: u32, base: &Base64, _pad: Padding, dest: *mut u8) -> *mut u8 {
	// let a = base.charset[((word >> 26) & 0x3F) as usize];
	// let b = base.charset[((word >> 20) & 0x3F) as usize];
	// let c = base.charset[((word >> 14) & 0x3F) as usize];
	// let d = base.charset[((word >> 8) & 0x3F) as usize];

	// (dest as *mut u32).write_unaligned(a as u32 | (b as u32) << 8 | (c as u32) << 16 | (d as u32) << 24);

	*dest.add(0) = base.charset[((word >> 26) & 0x3F) as usize];
	*dest.add(1) = base.charset[((word >> 20) & 0x3F) as usize];
	*dest.add(2) = base.charset[((word >> 14) & 0x3F) as usize];
	*dest.add(3) = base.charset[((word >> 8) & 0x3F) as usize];
	return dest.add(4);
}

// aaaaaabb bbbbcccc ccdddddd
#[inline]
unsafe fn encode_3bytes([b0, b1, b2]: &[u8; 3], base: &Base64, _pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 2) as usize];
	*dest.add(1) = base.charset[((b0 << 4 | b1 >> 4) & 0x3F) as usize];
	*dest.add(2) = base.charset[((b1 << 2 | b2 >> 6) & 0x3F) as usize];
	*dest.add(3) = base.charset[(b2 & 0x3F) as usize];

	return dest.add(4);
}

// aaaaaabb bbbbcccc cc------
#[inline]
unsafe fn encode_2bytes([b0, b1]: &[u8; 2], base: &Base64, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 2) as usize];
	*dest.add(1) = base.charset[((b0 << 4 | b1 >> 4) & 0x3F) as usize];
	*dest.add(2) = base.charset[(b1 << 2 & 0x3F) as usize];

	if matches!(pad, Padding::Strict) {
		*dest.add(3) = PAD_CHAR;
		dest.add(4)
	}
	else {
		dest.add(3)
	}
}

// aaaaaabb bbbbbb-- --------
#[inline]
unsafe fn encode_1byte([b0]: &[u8; 1], base: &Base64, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 2) as usize];
	*dest.add(1) = base.charset[(b0 << 4 & 0x3F) as usize];

	if matches!(pad, Padding::Strict) {
		*dest.add(2) = PAD_CHAR;
		*dest.add(3) = PAD_CHAR;
		dest.add(4)
	}
	else {
		dest.add(2)
	}
}

pub unsafe fn encode(mut bytes: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 4 {
		let word = (bytes.as_ptr() as *const u32).read_unaligned();
		#[cfg(target_endian = "little")]
		let word = word.swap_bytes();
		dest = encode_word(word, base, pad, dest);

		bytes = &bytes[3..];
	}

	// Encode remaining bytes
	dest = match bytes.len() {
		3 => encode_3bytes(&*(bytes.as_ptr() as *const [u8; 3]), base, pad, dest),
		2 => encode_2bytes(&*(bytes.as_ptr() as *const [u8; 2]), base, pad, dest),
		1 => encode_1byte(&*(bytes.as_ptr() as *const [u8; 1]), base, pad, dest),
		_ => dest,
	};

	return dest;
}
