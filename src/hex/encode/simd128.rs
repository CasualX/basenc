use core::arch::wasm32::*;

use super::*;

#[target_feature(enable = "simd128")]
pub unsafe fn encode(mut bytes: &[u8], mut dest: *mut u8, base: u8) -> *mut u8 {
	let mask = u8x16_splat(0x0f);
	let charset = if base == b'A' {
		u8x16(0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46)
	}
	else {
		u8x16(0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66)
	};

	while bytes.len() >= 16 {
		let data = v128_load(bytes.as_ptr() as *const v128);
		let lo = v128_and(data, mask);
		let hi = v128_and(u8x16_shr(data, 4), mask);
		let first = u8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(hi, lo);
		let second = u8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(hi, lo);
		v128_store(dest as *mut v128, u8x16_swizzle(charset, first));
		v128_store(dest.add(16) as *mut v128, u8x16_swizzle(charset, second));

		bytes = bytes.get_unchecked(16..);
		dest = dest.add(32);
	}

	scalar::encode(bytes, dest, base)
}
