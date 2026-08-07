use super::*;

// aaaaabbb bbcccccd ddddeeee efffffgg ggghhhhh
#[inline]
unsafe fn encode_5bytes([b0, b1, b2, b3, b4]: &[u8; 5], base: &Base32, _pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 3) as usize];
	*dest.add(1) = base.charset[((b0 << 2 | b1 >> 6) & 0x1F) as usize];
	*dest.add(2) = base.charset[(b1 >> 1 & 0x1F) as usize];
	*dest.add(3) = base.charset[((b1 << 4 | b2 >> 4) & 0x1F) as usize];
	*dest.add(4) = base.charset[((b2 << 1 | b3 >> 7) & 0x1F) as usize];
	*dest.add(5) = base.charset[(b3 >> 2 & 0x1F) as usize];
	*dest.add(6) = base.charset[((b3 << 3 | b4 >> 5) & 0x1F) as usize];
	*dest.add(7) = base.charset[(b4 & 0x1F) as usize];

	return dest.add(8);
}

// aaaaabbb bbcccccd ddddeeee efffffgg 000-----
#[inline]
unsafe fn encode_4bytes([b0, b1, b2, b3]: &[u8; 4], base: &Base32, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 3) as usize];
	*dest.add(1) = base.charset[((b0 << 2 | b1 >> 6) & 0x1F) as usize];
	*dest.add(2) = base.charset[(b1 >> 1 & 0x1F) as usize];
	*dest.add(3) = base.charset[((b1 << 4 | b2 >> 4) & 0x1F) as usize];
	*dest.add(4) = base.charset[((b2 << 1 | b3 >> 7) & 0x1F) as usize];
	*dest.add(5) = base.charset[(b3 >> 2 & 0x1F) as usize];
	*dest.add(6) = base.charset[((b3 << 3) & 0x1F) as usize];

	if pad.encode_padded() {
		*dest.add(7) = PAD_CHAR;
		dest.add(8)
	}
	else {
		dest.add(7)
	}
}

// aaaaabbb bbcccccd ddddeeee 0------- --------
#[inline]
unsafe fn encode_3bytes([b0, b1, b2]: &[u8; 3], base: &Base32, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 3) as usize];
	*dest.add(1) = base.charset[((b0 << 2 | b1 >> 6) & 0x1F) as usize];
	*dest.add(2) = base.charset[(b1 >> 1 & 0x1F) as usize];
	*dest.add(3) = base.charset[((b1 << 4 | b2 >> 4) & 0x1F) as usize];
	*dest.add(4) = base.charset[((b2 << 1) & 0x1F) as usize];

	if pad.encode_padded() {
		*dest.add(5) = PAD_CHAR;
		*dest.add(6) = PAD_CHAR;
		*dest.add(7) = PAD_CHAR;
		dest.add(8)
	}
	else {
		dest.add(5)
	}
}

// aaaaabbb bbcccccd 0000---- -------- --------
#[inline]
unsafe fn encode_2bytes([b0, b1]: &[u8; 2], base: &Base32, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 3) as usize];
	*dest.add(1) = base.charset[((b0 << 2 | b1 >> 6) & 0x1F) as usize];
	*dest.add(2) = base.charset[(b1 >> 1 & 0x1F) as usize];
	*dest.add(3) = base.charset[((b1 << 4) & 0x1F) as usize];

	if pad.encode_padded() {
		*dest.add(4) = PAD_CHAR;
		*dest.add(5) = PAD_CHAR;
		*dest.add(6) = PAD_CHAR;
		*dest.add(7) = PAD_CHAR;
		dest.add(8)
	}
	else {
		dest.add(4)
	}
}

// aaaaabbb 00------ -------- -------- --------
#[inline]
unsafe fn encode_1byte([b0]: &[u8; 1], base: &Base32, pad: Padding, dest: *mut u8) -> *mut u8 {
	*dest.add(0) = base.charset[(b0 >> 3) as usize];
	*dest.add(1) = base.charset[((b0 << 2) & 0x1F) as usize];

	if pad.encode_padded() {
		*dest.add(2) = PAD_CHAR;
		*dest.add(3) = PAD_CHAR;
		*dest.add(4) = PAD_CHAR;
		*dest.add(5) = PAD_CHAR;
		*dest.add(6) = PAD_CHAR;
		*dest.add(7) = PAD_CHAR;
		dest.add(8)
	}
	else {
		dest.add(2)
	}
}

pub unsafe fn encode(mut bytes: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while let [b0, b1, b2, b3, b4, ref rest @ ..] = *bytes {
		dest = encode_5bytes(&[b0, b1, b2, b3, b4], base, pad, dest);
		bytes = rest;
	}

	// Encode remaining bytes
	let dest = match *bytes {
		[b0, b1, b2, b3] => encode_4bytes(&[b0, b1, b2, b3], base, pad, dest),
		[b0, b1, b2] => encode_3bytes(&[b0, b1, b2], base, pad, dest),
		[b0, b1] => encode_2bytes(&[b0, b1], base, pad, dest),
		[b0] => encode_1byte(&[b0], base, pad, dest),
		_ => dest,
	};

	return dest;
}
