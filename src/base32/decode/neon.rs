use core::arch::aarch64::*;

use super::*;

#[inline]
#[target_feature(enable = "neon")]
unsafe fn lookup(input: uint8x16_t, base: &Base32) -> Result<uint8x16_t, crate::ErrorKind> {
	let lower_table = vld1q_u8_x4(base.lut.as_ptr());
	let upper_table = vld1q_u8_x4(base.lut.as_ptr().add(64));
	let lower = vqtbl4q_u8(lower_table, input);
	let upper = vqtbl4q_u8(upper_table, vsubq_u8(input, vdupq_n_u8(64)));
	let values = vbslq_u8(vcgeq_u8(input, vdupq_n_u8(64)), upper, lower);
	// TBL returns zero for an out-of-range index, so explicitly retain the
	// invalid marker for non-ASCII input instead of accepting it as index zero.
	let values = vorrq_u8(values, vcgeq_u8(input, vdupq_n_u8(128)));
	if vmaxvq_u8(values) > 31 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}
	Ok(values)
}

/// Pack two groups of eight five-bit indices into ten bytes.
#[inline]
#[target_feature(enable = "neon")]
unsafe fn pack(values: uint8x16_t) -> uint8x16_t {
	let first_indices = vld1q_u8([0, 1, 3, 4, 6, 8, 9, 11, 12, 14, 16, 16, 16, 16, 16, 16].as_ptr());
	let second_indices = vld1q_u8([1, 2, 4, 5, 7, 9, 10, 12, 13, 15, 16, 16, 16, 16, 16, 16].as_ptr());
	let third_indices = vld1q_u8([16, 3, 16, 6, 16, 16, 11, 16, 14, 16, 16, 16, 16, 16, 16, 16].as_ptr());
	let first_shifts = vld1q_s8([3, 6, 4, 7, 5, 3, 6, 4, 7, 5, 0, 0, 0, 0, 0, 0].as_ptr());
	let second_shifts = vld1q_s8([-2, 1, -1, 2, 0, -2, 1, -1, 2, 0, 0, 0, 0, 0, 0, 0].as_ptr());
	let third_shifts = vld1q_s8([0, -4, 0, -3, 0, 0, -4, 0, -3, 0, 0, 0, 0, 0, 0, 0].as_ptr());
	let first = vshlq_u8(vqtbl1q_u8(values, first_indices), first_shifts);
	let second = vshlq_u8(vqtbl1q_u8(values, second_indices), second_shifts);
	let third = vshlq_u8(vqtbl1q_u8(values, third_indices), third_shifts);
	vorrq_u8(first, vorrq_u8(second, third))
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn store(value: uint8x16_t, dest: *mut u8) {
	vst1_u8(dest, vget_low_u8(value));
	(dest.add(8) as *mut u16).write_unaligned(vgetq_lane_u16::<4>(vreinterpretq_u16_u8(value)));
}

#[target_feature(enable = "neon")]
pub unsafe fn decode(mut string: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	if string.len() < 32 {
		return scalar::decode(string, base, pad, dest);
	}

	while string.len() >= 16 {
		let Ok(values) = lookup(vld1q_u8(string.as_ptr()), base)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};
		store(pack(values), dest);
		string = string.get_unchecked(16..);
		dest = dest.add(10);
	}

	scalar::decode(string, base, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}
