use core::arch::aarch64::*;

use super::*;

#[inline]
#[target_feature(enable = "neon")]
unsafe fn decode_hex(input: uint8x16_t) -> Result<uint8x16_t, crate::ErrorKind> {
	let digit = vandq_u8(vcgeq_u8(input, vdupq_n_u8(b'0')), vcleq_u8(input, vdupq_n_u8(b'9')));
	let upper = vandq_u8(vcgeq_u8(input, vdupq_n_u8(b'A')), vcleq_u8(input, vdupq_n_u8(b'F')));
	let lower = vandq_u8(vcgeq_u8(input, vdupq_n_u8(b'a')), vcleq_u8(input, vdupq_n_u8(b'f')));
	let valid = vorrq_u8(digit, vorrq_u8(upper, lower));
	if vminvq_u8(valid) != u8::MAX {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let digit_value = vsubq_u8(input, vdupq_n_u8(b'0'));
	let upper_value = vaddq_u8(vsubq_u8(input, vdupq_n_u8(b'A')), vdupq_n_u8(10));
	let lower_value = vaddq_u8(vsubq_u8(input, vdupq_n_u8(b'a')), vdupq_n_u8(10));
	Ok(vbslq_u8(digit, digit_value, vbslq_u8(upper, upper_value, lower_value)))
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn nibbles_to_bytes(nibbles: uint8x16_t) -> uint8x8_t {
	let high = vget_low_u8(vuzp1q_u8(nibbles, vdupq_n_u8(0)));
	let low = vget_low_u8(vuzp2q_u8(nibbles, vdupq_n_u8(0)));
	vorr_u8(vshl_n_u8::<4>(high), low)
}

#[target_feature(enable = "neon")]
pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 32 {
		let first = decode_hex(vld1q_u8(string.as_ptr()));
		let second = decode_hex(vld1q_u8(string.as_ptr().add(16)));
		let (Ok(first), Ok(second)) = (first, second)
		else {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		};
		let bytes = vcombine_u8(nibbles_to_bytes(first), nibbles_to_bytes(second));
		vst1q_u8(dest, bytes);
		string = string.get_unchecked(32..);
		dest = dest.add(16);
	}

	if string.len() >= 16 {
		let Ok(nibbles) = decode_hex(vld1q_u8(string.as_ptr()))
		else {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		};
		vst1_u8(dest, nibbles_to_bytes(nibbles));
		string = string.get_unchecked(16..);
		dest = dest.add(8);
	}

	scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()))
}
