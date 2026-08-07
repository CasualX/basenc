use core::arch::wasm32::*;

use super::*;

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn lookup(src: *const u8, base: &Base64) -> Result<v128, crate::ErrorKind> {
	let input = v128_load(src as *const v128);
	let upper = v128_and(u8x16_ge(input, u8x16_splat(b'A')), u8x16_le(input, u8x16_splat(b'Z')));
	let lower = v128_and(u8x16_ge(input, u8x16_splat(b'a')), u8x16_le(input, u8x16_splat(b'z')));
	let digit = v128_and(u8x16_ge(input, u8x16_splat(b'0')), u8x16_le(input, u8x16_splat(b'9')));
	let is62 = u8x16_eq(input, u8x16_splat(base.charset[62]));
	let is63 = u8x16_eq(input, u8x16_splat(base.charset[63]));
	let valid = v128_or(v128_or(upper, lower), v128_or(digit, v128_or(is62, is63)));
	if u8x16_bitmask(valid) != 0xffff {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let upper_value = u8x16_sub(input, u8x16_splat(b'A'));
	let lower_value = u8x16_add(u8x16_sub(input, u8x16_splat(b'a')), u8x16_splat(26));
	let digit_value = u8x16_add(u8x16_sub(input, u8x16_splat(b'0')), u8x16_splat(52));
	let values = v128_bitselect(lower_value, upper_value, lower);
	let values = v128_bitselect(digit_value, values, digit);
	let values = v128_bitselect(u8x16_splat(62), values, is62);
	Ok(v128_bitselect(u8x16_splat(63), values, is63))
}

#[inline]
#[target_feature(enable = "simd128")]
fn pack(values: v128) -> v128 {
	let b0 = v128_or(
		i32x4_shl(v128_and(values, u32x4_splat(0x0000_003f)), 2),
		u32x4_shr(v128_and(values, u32x4_splat(0x0000_3000)), 12),
	);
	let b1 = v128_or(
		i32x4_shl(v128_and(values, u32x4_splat(0x0000_0f00)), 4),
		u32x4_shr(v128_and(values, u32x4_splat(0x003c_0000)), 10),
	);
	let b2 = v128_or(
		i32x4_shl(v128_and(values, u32x4_splat(0x0003_0000)), 6),
		u32x4_shr(v128_and(values, u32x4_splat(0x3f00_0000)), 8),
	);
	let packed = v128_or(b0, v128_or(b1, b2));
	u8x16_shuffle::<0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16, 16, 16, 16>(packed, u8x16_splat(0))
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn store(value: v128, dest: *mut u8) {
	v128_store64_lane::<0>(value, dest as *mut u64);
	v128_store32_lane::<2>(value, dest.add(8) as *mut u32);
}

#[target_feature(enable = "simd128")]
pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 16 {
		let Ok(values) = lookup(string.as_ptr(), base)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};
		store(pack(values), dest);
		string = string.get_unchecked(16..);
		dest = dest.add(12);
	}
	scalar::decode(string, base, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}
