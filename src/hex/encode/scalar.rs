
fn encode_char(nibble: u8, base: u8) -> u8 {
	nibble + if nibble < 10 { b'0' } else { base - 10 }
}

pub unsafe fn encode(mut bytes: &[u8], mut dest: *mut u8, base: u8) -> *mut u8 {
	while bytes.len() > 0 {
		let byte = bytes[0];
		let hi = byte >> 4;
		let lo = byte & 0xF;

		*dest.offset(0) = encode_char(hi, base);
		*dest.offset(1) = encode_char(lo, base);

		dest = dest.offset(2);
		bytes = &bytes[1..];
	}

	return dest;
}
