use core::arch::aarch64::*;

use super::*;

/// Load exactly ten bytes, leaving the other six lanes initialized to zero.
#[inline]
#[target_feature(enable = "neon")]
unsafe fn load_10(bytes: *const u8) -> uint8x16_t {
	let mut input = vcombine_u8(vld1_u8(bytes), vdup_n_u8(0));
	input = vsetq_lane_u8::<8>(*bytes.add(8), input);
	vsetq_lane_u8::<9>(*bytes.add(9), input)
}

/// Split two five-byte blocks into sixteen five-bit indices.
#[inline]
#[target_feature(enable = "neon")]
unsafe fn split(input: uint8x16_t) -> uint8x16_t {
	let first_indices = vld1q_u8([0, 0, 1, 1, 2, 3, 3, 4, 5, 5, 6, 6, 7, 8, 8, 9].as_ptr());
	let second_indices = vld1q_u8([16, 1, 16, 2, 3, 16, 4, 16, 16, 6, 16, 7, 8, 16, 9, 16].as_ptr());
	let first_shifts = vld1q_s8([-3, 2, -1, 4, 1, -2, 3, 0, -3, 2, -1, 4, 1, -2, 3, 0].as_ptr());
	let second_shifts = vld1q_s8([0, -6, 0, -4, -7, 0, -5, 0, 0, -6, 0, -4, -7, 0, -5, 0].as_ptr());
	let first = vshlq_u8(vqtbl1q_u8(input, first_indices), first_shifts);
	let second = vshlq_u8(vqtbl1q_u8(input, second_indices), second_shifts);
	vandq_u8(vorrq_u8(first, second), vdupq_n_u8(0x1f))
}

/// Translate arbitrary indices through the caller-provided alphabet.
#[inline]
#[target_feature(enable = "neon")]
unsafe fn lookup(indices: uint8x16_t, base: &Base32) -> uint8x16_t {
	vqtbl2q_u8(vld1q_u8_x2(base.charset.as_ptr()), indices)
}

#[target_feature(enable = "neon")]
pub unsafe fn encode(mut bytes: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 16 {
		vst1q_u8(dest, lookup(split(vld1q_u8(bytes.as_ptr())), base));
		bytes = bytes.get_unchecked(10..);
		dest = dest.add(16);
	}

	while bytes.len() >= 10 {
		vst1q_u8(dest, lookup(split(load_10(bytes.as_ptr())), base));
		bytes = bytes.get_unchecked(10..);
		dest = dest.add(16);
	}

	scalar::encode(bytes, base, pad, dest)
}
