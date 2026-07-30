#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn split_bytes(value: __m256i) -> __m256i {
	let input = _mm256_shuffle_epi8(value, _mm256_setr_epi8(
		1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8, 7, 10, 9, 11, 10,
		1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8, 7, 10, 9, 11, 10,
	));
	let t0 = _mm256_and_si256(input, _mm256_set1_epi32(0x0fc0fc00));
	let t1 = _mm256_mulhi_epu16(t0, _mm256_set1_epi32(0x04000040));
	let t2 = _mm256_and_si256(input, _mm256_set1_epi32(0x003f03f0));
	let t3 = _mm256_mullo_epi16(t2, _mm256_set1_epi32(0x01000010));
	_mm256_or_si256(t1, t3)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn lookup(input: __m256i) -> __m256i {
	let result = _mm256_subs_epu8(input, _mm256_set1_epi8(51));
	let less = _mm256_cmpgt_epi8(_mm256_set1_epi8(26), input);
	let result = _mm256_or_si256(result, _mm256_and_si256(less, _mm256_set1_epi8(13)));
	let shift_lut = _mm256_setr_epi8(
		71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 65, 0, 0,
		71, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -19, -16, 65, 0, 0,
	);
	_mm256_add_epi8(_mm256_shuffle_epi8(shift_lut, result), input)
}

#[target_feature(enable = "avx2")]
pub unsafe fn encode(mut bytes: &[u8], pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 28 {
		let low = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
		let high = _mm_loadu_si128(bytes.as_ptr().add(12) as *const __m128i);
		let data = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(low), high);
		_mm256_storeu_si256(dest as *mut __m256i, lookup(split_bytes(data)));
		bytes = bytes.get_unchecked(24..);
		dest = dest.add(32);
	}

	if bytes.len() >= 16 {
		let data = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
		let lanes = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(data), data);
		let ascii = lookup(split_bytes(lanes));
		_mm_storeu_si128(dest as *mut __m128i, _mm256_castsi256_si128(ascii));
		bytes = bytes.get_unchecked(12..);
		dest = dest.add(16);
	}

	scalar::encode(bytes, pad, dest)
}
