use core::arch::aarch64::*;

use super::*;

#[target_feature(enable = "neon")]
pub unsafe fn encode(mut bytes: &[u8], mut dest: *mut u8, base: u8) -> *mut u8 {
	let mask = vdupq_n_u8(0x0f);
	let charset = if base == b'A' {
		vld1q_u8(b"0123456789ABCDEF".as_ptr())
	}
	else {
		vld1q_u8(b"0123456789abcdef".as_ptr())
	};

	while bytes.len() >= 16 {
		let data = vld1q_u8(bytes.as_ptr());
		let low = vandq_u8(data, mask);
		let high = vandq_u8(vshrq_n_u8::<4>(data), mask);
		let first = vzip1q_u8(high, low);
		let second = vzip2q_u8(high, low);
		vst1q_u8(dest, vqtbl1q_u8(charset, first));
		vst1q_u8(dest.add(16), vqtbl1q_u8(charset, second));

		bytes = bytes.get_unchecked(16..);
		dest = dest.add(32);
	}

	scalar::encode(bytes, dest, base)
}
