use core::arch::wasm32::*;

use super::*;

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn encode_block(src: *const u8, base: &Base64) -> v128 {
	let input = v128_load(src as *const v128);
	let zero = u8x16_splat(0);
	let words = u8x16_shuffle::<2, 1, 0, 16, 5, 4, 3, 16, 8, 7, 6, 16, 11, 10, 9, 16>(input, zero);
	let mask = u32x4_splat(0x3f);
	let i0 = u32x4_shr(words, 18);
	let i1 = i32x4_shl(v128_and(u32x4_shr(words, 12), mask), 8);
	let i2 = i32x4_shl(v128_and(u32x4_shr(words, 6), mask), 16);
	let i3 = i32x4_shl(v128_and(words, mask), 24);
	lookup(v128_or(v128_or(i0, i1), v128_or(i2, i3)), base)
}

#[inline]
#[target_feature(enable = "simd128")]
fn lookup(indices: v128, base: &Base64) -> v128 {
	let upper = u8x16_add(indices, u8x16_splat(b'A'));
	let lower = u8x16_add(indices, u8x16_splat(b'a' - 26));
	let digits = u8x16_add(indices, u8x16_splat(b'0'.wrapping_sub(52)));
	let lower_mask = u8x16_ge(indices, u8x16_splat(26));
	let digit_mask = u8x16_ge(indices, u8x16_splat(52));
	let char62_mask = u8x16_eq(indices, u8x16_splat(62));
	let char63_mask = u8x16_eq(indices, u8x16_splat(63));
	let ascii = v128_bitselect(lower, upper, lower_mask);
	let ascii = v128_bitselect(digits, ascii, digit_mask);
	let ascii = v128_bitselect(u8x16_splat(base.charset[62]), ascii, char62_mask);
	v128_bitselect(u8x16_splat(base.charset[63]), ascii, char63_mask)
}

#[target_feature(enable = "simd128")]
pub unsafe fn encode(mut bytes: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 16 {
		v128_store(dest as *mut v128, encode_block(bytes.as_ptr(), base));
		bytes = bytes.get_unchecked(12..);
		dest = dest.add(16);
	}
	scalar::encode(bytes, base, pad, dest)
}
