use core::arch::wasm32::*;

use super::*;

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn decode_hex(input: v128) -> Result<v128, crate::ErrorKind> {
	let digit = v128_and(u8x16_ge(input, u8x16_splat(b'0')), u8x16_le(input, u8x16_splat(b'9')));
	let upper = v128_and(u8x16_ge(input, u8x16_splat(b'A')), u8x16_le(input, u8x16_splat(b'F')));
	let lower = v128_and(u8x16_ge(input, u8x16_splat(b'a')), u8x16_le(input, u8x16_splat(b'f')));
	let valid = v128_or(digit, v128_or(upper, lower));
	if u8x16_bitmask(valid) != 0xffff {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let digit_value = u8x16_sub(input, u8x16_splat(b'0'));
	let upper_value = u8x16_add(u8x16_sub(input, u8x16_splat(b'A')), u8x16_splat(10));
	let lower_value = u8x16_add(u8x16_sub(input, u8x16_splat(b'a')), u8x16_splat(10));
	Ok(v128_bitselect(digit_value, v128_bitselect(upper_value, lower_value, upper), digit))
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn nibbles_to_bytes(nibbles: v128) -> v128 {
	let zero = u8x16_splat(0);
	let hi = u8x16_shuffle::<0, 2, 4, 6, 8, 10, 12, 14, 16, 16, 16, 16, 16, 16, 16, 16>(nibbles, zero);
	let lo = u8x16_shuffle::<1, 3, 5, 7, 9, 11, 13, 15, 16, 16, 16, 16, 16, 16, 16, 16>(nibbles, zero);
	v128_or(u8x16_shl(hi, 4), lo)
}

#[target_feature(enable = "simd128")]
pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 32 {
		let first = decode_hex(v128_load(string.as_ptr() as *const v128));
		let second = decode_hex(v128_load(string.as_ptr().add(16) as *const v128));
		let (Ok(first), Ok(second)) = (first, second)
		else {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		};
		let bytes = u8x16_shuffle::<0, 1, 2, 3, 4, 5, 6, 7, 16, 17, 18, 19, 20, 21, 22, 23>(
			nibbles_to_bytes(first),
			nibbles_to_bytes(second),
		);
		v128_store(dest as *mut v128, bytes);
		string = string.get_unchecked(32..);
		dest = dest.add(16);
	}

	if string.len() >= 16 {
		let Ok(nibbles) = decode_hex(v128_load(string.as_ptr() as *const v128))
		else {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		};
		let bytes = nibbles_to_bytes(nibbles);
		v128_store64_lane::<0>(bytes, dest as *mut u64);
		string = string.get_unchecked(16..);
		dest = dest.add(8);
	}

	scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()))
}
