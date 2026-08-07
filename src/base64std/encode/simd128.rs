use core::arch::wasm32::*;

use super::*;

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn encode_block(src: *const u8) -> v128 {
	let input = v128_load(src as *const v128);
	let zero = u8x16_splat(0);
	let words = u8x16_shuffle::<2, 1, 0, 16, 5, 4, 3, 16, 8, 7, 6, 16, 11, 10, 9, 16>(input, zero);
	let mask = u32x4_splat(0x3f);
	let i0 = u32x4_shr(words, 18);
	let i1 = i32x4_shl(v128_and(u32x4_shr(words, 12), mask), 8);
	let i2 = i32x4_shl(v128_and(u32x4_shr(words, 6), mask), 16);
	let i3 = i32x4_shl(v128_and(words, mask), 24);
	lookup(v128_or(v128_or(i0, i1), v128_or(i2, i3)))
}

#[inline]
#[target_feature(enable = "simd128")]
fn lookup(indices: v128) -> v128 {
	// Indices 0..25 select the final entry, 26..51 the first, and
	// 52..63 entries 1..12. Each table byte is the ASCII adjustment.
	let reduced = u8x16_sub_sat(indices, u8x16_splat(51));
	let upper = v128_and(u8x16_lt(indices, u8x16_splat(26)), u8x16_splat(13));
	let selector = v128_or(reduced, upper);
	let shifts = i8x16(71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 65, 0, 0);
	u8x16_add(indices, i8x16_swizzle(shifts, selector))
}

#[target_feature(enable = "simd128")]
pub unsafe fn encode(mut bytes: &[u8], pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 16 {
		v128_store(dest as *mut v128, encode_block(bytes.as_ptr()));
		bytes = bytes.get_unchecked(12..);
		dest = dest.add(16);
	}
	scalar::encode(bytes, pad, dest)
}
