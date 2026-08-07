use core::arch::aarch64::*;

use super::*;

#[inline]
#[target_feature(enable = "neon")]
unsafe fn lookup(src: *const u8, base: &Base64) -> Result<uint8x16_t, crate::ErrorKind> {
	let input = vld1q_u8(src);
	let upper = vandq_u8(vcgeq_u8(input, vdupq_n_u8(b'A')), vcleq_u8(input, vdupq_n_u8(b'Z')));
	let lower = vandq_u8(vcgeq_u8(input, vdupq_n_u8(b'a')), vcleq_u8(input, vdupq_n_u8(b'z')));
	let digit = vandq_u8(vcgeq_u8(input, vdupq_n_u8(b'0')), vcleq_u8(input, vdupq_n_u8(b'9')));
	let is62 = vceqq_u8(input, vdupq_n_u8(base.charset[62]));
	let is63 = vceqq_u8(input, vdupq_n_u8(base.charset[63]));
	let valid = vorrq_u8(vorrq_u8(upper, lower), vorrq_u8(digit, vorrq_u8(is62, is63)));
	if vminvq_u8(valid) != u8::MAX {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let upper_value = vsubq_u8(input, vdupq_n_u8(b'A'));
	let lower_value = vaddq_u8(vsubq_u8(input, vdupq_n_u8(b'a')), vdupq_n_u8(26));
	let digit_value = vaddq_u8(vsubq_u8(input, vdupq_n_u8(b'0')), vdupq_n_u8(52));
	let values = vbslq_u8(lower, lower_value, upper_value);
	let values = vbslq_u8(digit, digit_value, values);
	let values = vbslq_u8(is62, vdupq_n_u8(62), values);
	Ok(vbslq_u8(is63, vdupq_n_u8(63), values))
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn pack(values: uint8x16_t) -> uint8x16_t {
	let values = vreinterpretq_u32_u8(values);
	let b0 = vorrq_u32(
		vshlq_n_u32::<2>(vandq_u32(values, vdupq_n_u32(0x0000_003f))),
		vshrq_n_u32::<12>(vandq_u32(values, vdupq_n_u32(0x0000_3000))),
	);
	let b1 = vorrq_u32(
		vshlq_n_u32::<4>(vandq_u32(values, vdupq_n_u32(0x0000_0f00))),
		vshrq_n_u32::<10>(vandq_u32(values, vdupq_n_u32(0x003c_0000))),
	);
	let b2 = vorrq_u32(
		vshlq_n_u32::<6>(vandq_u32(values, vdupq_n_u32(0x0003_0000))),
		vshrq_n_u32::<8>(vandq_u32(values, vdupq_n_u32(0x3f00_0000))),
	);
	let packed = vreinterpretq_u8_u32(vorrq_u32(b0, vorrq_u32(b1, b2)));
	let compact = vld1q_u8([0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16, 16, 16, 16].as_ptr());
	vqtbl1q_u8(packed, compact)
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn store(value: uint8x16_t, dest: *mut u8) {
	vst1_u8(dest, vget_low_u8(value));
	(dest.add(8) as *mut u32).write_unaligned(vgetq_lane_u32::<2>(vreinterpretq_u32_u8(value)));
}

#[target_feature(enable = "neon")]
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
