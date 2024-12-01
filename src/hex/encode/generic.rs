
fn encode_char(nibble: u8, base: u8) -> u8 {
	nibble + if nibble < 10 { b'0' } else { base }
}

pub unsafe fn encode(src: *const u8, len: usize, mut dest: *mut u8, base: u8) {
	let base = base - 10;

	let bytes = core::slice::from_raw_parts(src, len);

	for &byte in bytes {
		let hi = byte >> 4;
		let lo = byte & 0xF;

		*dest = encode_char(hi, base);
		dest = dest.offset(1);
		*dest = encode_char(lo, base);
		dest = dest.offset(1);
	}
}
