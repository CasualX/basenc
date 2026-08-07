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

// aaaaaabb bbbbcccc 00------
#[inline]
unsafe fn encode_2bytes([b0, b1]: &[u8; 2], base: &Base64, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 2) as usize];
	*dest.add(1) = base.charset[((b0 << 4 | b1 >> 4) & 0x3F) as usize];
	*dest.add(2) = base.charset[(b1 << 2 & 0x3F) as usize];

	if pad.encode_padded() {
		*dest.add(3) = PAD_CHAR;
		dest.add(4)
	}
	else {
		dest.add(3)
	}
}

// aaaaaabb 0000---- --------
#[inline]
unsafe fn encode_1byte([b0]: &[u8; 1], base: &Base64, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 2) as usize];
	*dest.add(1) = base.charset[(b0 << 4 & 0x3F) as usize];

	if pad.encode_padded() {
		*dest.add(2) = PAD_CHAR;
		*dest.add(3) = PAD_CHAR;
		dest.add(4)
	}
	else {
		dest.add(2)
	}
}

pub unsafe fn encode(mut bytes: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	// Read 4 bytes at a time as a big-endian u32, but only consume 3 bytes per iteration.
	// This lets `encode_word` extract four 6-bit groups via simple shifts on the upper 24 bits
	// (bits 26..31, 20..25, 14..19, 8..13) without manually packing 3 separate bytes.
	// The lowest 8 bits (the 4th byte) are ignored and will be re-read in the next iteration.
	// Requires len >= 4 so the u32 read doesn't go out of bounds.
	while bytes.len() >= 4 {
		let word = (bytes.as_ptr() as *const u32).read_unaligned();
		#[cfg(target_endian = "little")]
		let word = word.swap_bytes();
		dest = encode_word(word, base, pad, dest);

		bytes = &bytes[3..];
	}

	// Encode remaining bytes
	dest = match *bytes {
		[b0, b1, b2] => encode_3bytes(&[b0, b1, b2], base, pad, dest),
		[b0, b1] => encode_2bytes(&[b0, b1], base, pad, dest),
		[b0] => encode_1byte(&[b0], base, pad, dest),
		_ => dest,
	};

	return dest;
}
