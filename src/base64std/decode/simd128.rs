use core::arch::wasm32::*;

use super::*;

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn lookup(src: *const u8) -> Result<v128, crate::ErrorKind> {
	let input = v128_load(src as *const v128);
	let higher = v128_and(u8x16_shr(input, 4), u8x16_splat(0x0f));
	let lower = v128_and(input, u8x16_splat(0x0f));
	let lower_lut = u8x16(0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1a, 0x1b, 0x1b, 0x1b, 0x1a);
	let higher_lut = u8x16(0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10);
	let invalid = v128_and(u8x16_swizzle(lower_lut, lower), u8x16_swizzle(higher_lut, higher));
	if u8x16_bitmask(u8x16_eq(invalid, u8x16_splat(0))) != 0xffff {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let slash = u8x16_eq(input, u8x16_splat(b'/'));
	let selector = u8x16_add(higher, slash);
	let offsets = i8x16(0, 16, 19, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0);
	Ok(u8x16_add(input, i8x16_swizzle(offsets, selector)))
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
pub unsafe fn decode(mut string: &[u8], pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 16 {
		let Ok(values) = lookup(string.as_ptr())
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};
		store(pack(values), dest);
		string = string.get_unchecked(16..);
		dest = dest.add(12);
	}
	scalar::decode(string, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}
