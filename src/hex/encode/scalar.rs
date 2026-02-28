
fn encode_char(nibble: u8, base: u8) -> u8 {
	nibble + if nibble < 10 { b'0' } else { base - 10 }
}

pub unsafe fn encode(mut bytes: &[u8], mut dest: *mut u8, base: u8) -> *mut u8 {
	while let &[byte, ref rest @ ..] = bytes {
		let hi = byte >> 4;
		let lo = byte & 0xF;

		*dest.add(0) = encode_char(hi, base);
		*dest.add(1) = encode_char(lo, base);

		dest = dest.add(2);
		bytes = rest;
	}

	return dest;
}
