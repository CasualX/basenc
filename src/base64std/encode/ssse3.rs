#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn split_bytes(value: __m128i) -> __m128i {
	let input = _mm_shuffle_epi8(value, _mm_setr_epi8(
		1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8, 7, 10, 9, 11, 10,
	));
	let t0 = _mm_and_si128(input, _mm_set1_epi32(0x0fc0fc00));
	let t1 = _mm_mulhi_epu16(t0, _mm_set1_epi32(0x04000040));
	let t2 = _mm_and_si128(input, _mm_set1_epi32(0x003f03f0));
	let t3 = _mm_mullo_epi16(t2, _mm_set1_epi32(0x01000010));
	_mm_or_si128(t1, t3)
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(input: __m128i) -> __m128i {
	let result = _mm_subs_epu8(input, _mm_set1_epi8(51));
	let less = _mm_cmpgt_epi8(_mm_set1_epi8(26), input);
	let result = _mm_or_si128(result, _mm_and_si128(less, _mm_set1_epi8(13)));
	let shift_lut = _mm_setr_epi8(
		71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 65, 0, 0,
	);
	_mm_add_epi8(_mm_shuffle_epi8(shift_lut, result), input)
}

#[target_feature(enable = "ssse3")]
pub unsafe fn encode(mut bytes: &[u8], pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 16 {
		let data = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
		_mm_storeu_si128(dest as *mut __m128i, lookup(split_bytes(data)));
		bytes = bytes.get_unchecked(12..);
		dest = dest.add(16);
	}
	scalar::encode(bytes, pad, dest)
}
