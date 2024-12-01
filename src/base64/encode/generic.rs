use super::*;

pub unsafe fn encode(mut bytes: &[u8], base: &Base64, mut buf: *mut u8) {
	// Encode 3 bytes into 4 characters
	while let &[b0, b1, b2, ref tail @ ..] = bytes {
		bytes = tail;

		*buf.add(0) = base.charset[(b0 >> 2) as usize];
		*buf.add(1) = base.charset[((b0 << 4 | b1 >> 4) & 0x3F) as usize];
		*buf.add(2) = base.charset[((b1 << 2 | b2 >> 6) & 0x3F) as usize];
		*buf.add(3) = base.charset[(b2 & 0x3F) as usize];
		buf = buf.add(4);
	}

	// Encode remaining 1 or 2 bytes
	match bytes {
		&[b0] => {
			*buf.add(0) = base.charset[(b0 >> 2) as usize];
			*buf.add(1) = base.charset[(b0 << 4 & 0x3F) as usize];
			if matches!(base.padding, Padding::Strict) {
				*buf.add(2) = PAD_CHAR;
				*buf.add(3) = PAD_CHAR;
			}
		},
		&[b0, b1] => {
			*buf.add(0) = base.charset[(b0 >> 2) as usize];
			*buf.add(1) = base.charset[((b0 << 4 | b1 >> 4) & 0x3F) as usize];
			*buf.add(2) = base.charset[(b1 << 2 & 0x3F) as usize];
			if matches!(base.padding, Padding::Strict) {
				*buf.add(3) = PAD_CHAR;
			}
		},
		_ => {},
	}
}
