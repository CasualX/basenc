use super::*;

#[inline]
unsafe fn encode_word(word: u32, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = CHARSET[((word >> 26) & 0x3f) as usize];
	*dest.add(1) = CHARSET[((word >> 20) & 0x3f) as usize];
	*dest.add(2) = CHARSET[((word >> 14) & 0x3f) as usize];
	*dest.add(3) = CHARSET[((word >> 8) & 0x3f) as usize];
	dest.add(4)
}

#[inline]
unsafe fn encode_3bytes([b0, b1, b2]: &[u8; 3], dest: *mut u8) -> *mut u8 {
	*dest.add(0) = CHARSET[(b0 >> 2) as usize];
	*dest.add(1) = CHARSET[((b0 << 4 | b1 >> 4) & 0x3f) as usize];
	*dest.add(2) = CHARSET[((b1 << 2 | b2 >> 6) & 0x3f) as usize];
	*dest.add(3) = CHARSET[(b2 & 0x3f) as usize];
	dest.add(4)
}

#[inline]
unsafe fn encode_2bytes([b0, b1]: &[u8; 2], pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = CHARSET[(b0 >> 2) as usize];
	*dest.add(1) = CHARSET[((b0 << 4 | b1 >> 4) & 0x3f) as usize];
	*dest.add(2) = CHARSET[(b1 << 2 & 0x3f) as usize];

	if pad.encode_padded() {
		*dest.add(3) = PAD_CHAR;
		dest.add(4)
	}
	else {
		dest.add(3)
	}
}

#[inline]
unsafe fn encode_1byte([b0]: &[u8; 1], pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = CHARSET[(b0 >> 2) as usize];
	*dest.add(1) = CHARSET[(b0 << 4 & 0x3f) as usize];

	if pad.encode_padded() {
		*dest.add(2) = PAD_CHAR;
		*dest.add(3) = PAD_CHAR;
		dest.add(4)
	}
	else {
		dest.add(2)
	}
}

pub unsafe fn encode(mut bytes: &[u8], pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 4 {
		let word = (bytes.as_ptr() as *const u32).read_unaligned();
		#[cfg(target_endian = "little")]
		let word = word.swap_bytes();
		dest = encode_word(word, dest);
		bytes = bytes.get_unchecked(3..);
	}

	match *bytes {
		[b0, b1, b2] => encode_3bytes(&[b0, b1, b2], dest),
		[b0, b1] => encode_2bytes(&[b0, b1], pad, dest),
		[b0] => encode_1byte(&[b0], pad, dest),
		_ => dest,
	}
}
