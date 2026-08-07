use core::arch::aarch64::*;

use super::*;

#[inline]
#[target_feature(enable = "neon")]
unsafe fn encode_block(src: *const u8, base: &Base64) -> uint8x16_t {
	let input = vld1q_u8(src);
	let reorder = vld1q_u8([2, 1, 0, 16, 5, 4, 3, 16, 8, 7, 6, 16, 11, 10, 9, 16].as_ptr());
	let words = vreinterpretq_u32_u8(vqtbl1q_u8(input, reorder));
	let mask = vdupq_n_u32(0x3f);
	let i0 = vshrq_n_u32::<18>(words);
	let i1 = vshlq_n_u32::<8>(vandq_u32(vshrq_n_u32::<12>(words), mask));
	let i2 = vshlq_n_u32::<16>(vandq_u32(vshrq_n_u32::<6>(words), mask));
	let i3 = vshlq_n_u32::<24>(vandq_u32(words, mask));
	lookup(vreinterpretq_u8_u32(vorrq_u32(vorrq_u32(i0, i1), vorrq_u32(i2, i3))), base)
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn lookup(indices: uint8x16_t, base: &Base64) -> uint8x16_t {
	let upper = vaddq_u8(indices, vdupq_n_u8(b'A'));
	let lower = vaddq_u8(indices, vdupq_n_u8(b'a' - 26));
	let digits = vaddq_u8(indices, vdupq_n_u8(b'0'.wrapping_sub(52)));
	let lower_mask = vcgeq_u8(indices, vdupq_n_u8(26));
	let digit_mask = vcgeq_u8(indices, vdupq_n_u8(52));
	let char62_mask = vceqq_u8(indices, vdupq_n_u8(62));
	let char63_mask = vceqq_u8(indices, vdupq_n_u8(63));
	let ascii = vbslq_u8(lower_mask, lower, upper);
	let ascii = vbslq_u8(digit_mask, digits, ascii);
	let ascii = vbslq_u8(char62_mask, vdupq_n_u8(base.charset[62]), ascii);
	vbslq_u8(char63_mask, vdupq_n_u8(base.charset[63]), ascii)
}

#[target_feature(enable = "neon")]
pub unsafe fn encode(mut bytes: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 16 {
		vst1q_u8(dest, encode_block(bytes.as_ptr(), base));
		bytes = bytes.get_unchecked(12..);
		dest = dest.add(16);
	}
	scalar::encode(bytes, base, pad, dest)
}
