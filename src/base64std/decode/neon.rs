use core::arch::aarch64::*;

use super::*;

#[inline]
#[target_feature(enable = "neon")]
unsafe fn lookup(src: *const u8) -> Result<uint8x16_t, crate::ErrorKind> {
	let input = vld1q_u8(src);
	let higher = vandq_u8(vshrq_n_u8::<4>(input), vdupq_n_u8(0x0f));
	let lower = vandq_u8(input, vdupq_n_u8(0x0f));
	let lower_lut = vld1q_u8([0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1a, 0x1b, 0x1b, 0x1b, 0x1a].as_ptr());
	let higher_lut = vld1q_u8([0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10].as_ptr());
	let invalid = vandq_u8(vqtbl1q_u8(lower_lut, lower), vqtbl1q_u8(higher_lut, higher));
	if vmaxvq_u8(invalid) != 0 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let slash = vceqq_u8(input, vdupq_n_u8(b'/'));
	let selector = vaddq_u8(higher, slash);
	let offsets = vld1q_s8([0, 16, 19, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr());
	Ok(vaddq_u8(input, vreinterpretq_u8_s8(vqtbl1q_s8(offsets, selector))))
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
