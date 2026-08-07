#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn load_10(bytes: *const u8) -> __m128i {
	let low = _mm_loadl_epi64(bytes as *const __m128i);
	_mm_insert_epi16::<4>(low, (bytes.add(8) as *const u16).read_unaligned() as i32)
}

/// Split four five-byte blocks into thirty-two five-bit indices.
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn split(input: __m256i) -> __m256i {
	let first = _mm256_shuffle_epi8(input, _mm256_setr_epi8(
		1, 0, 1, 0, 2, 1, 2, 1, 3, 2, 4, 3, 4, 3, 4, 4,
		1, 0, 1, 0, 2, 1, 2, 1, 3, 2, 4, 3, 4, 3, 4, 4,
	));
	let second = _mm256_shuffle_epi8(input, _mm256_setr_epi8(
		6, 5, 6, 5, 7, 6, 7, 6, 8, 7, 9, 8, 9, 8, 9, 9,
		6, 5, 6, 5, 7, 6, 7, 6, 8, 7, 9, 8, 9, 8, 9, 9,
	));
	let shifts = _mm256_setr_epi16(
		32, 1024, 128, 4096, 512, 64, 2048, 256,
		32, 1024, 128, 4096, 512, 64, 2048, 256,
	);
	let mask = _mm256_set1_epi16(0x1f);
	let first = _mm256_and_si256(_mm256_mulhi_epu16(first, shifts), mask);
	let second = _mm256_and_si256(_mm256_mulhi_epu16(second, shifts), mask);
	_mm256_packus_epi16(first, second)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn lookup(indices: __m256i, base: &Base32) -> __m256i {
	let low = _mm_loadu_si128(base.charset.as_ptr() as *const __m128i);
	let high = _mm_loadu_si128(base.charset.as_ptr().add(16) as *const __m128i);
	let low = _mm256_broadcastsi128_si256(low);
	let high = _mm256_broadcastsi128_si256(high);
	let high_mask = _mm256_cmpgt_epi8(indices, _mm256_set1_epi8(15));
	let low = _mm256_shuffle_epi8(low, indices);
	let high = _mm256_shuffle_epi8(high, indices);
	_mm256_or_si256(_mm256_andnot_si256(high_mask, low), _mm256_and_si256(high_mask, high))
}

#[target_feature(enable = "avx2")]
pub unsafe fn encode(mut bytes: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	// Each lane consumes ten bytes. With 26 readable bytes, two ordinary
	// 16-byte loads are safe and cheaper than constructing exact-width lanes.
	while bytes.len() >= 26 {
		let low = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
		let high = _mm_loadu_si128(bytes.as_ptr().add(10) as *const __m128i);
		let input = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(low), high);
		_mm256_storeu_si256(dest as *mut __m256i, lookup(split(input), base));
		bytes = bytes.get_unchecked(20..);
		dest = dest.add(32);
	}

	while bytes.len() >= 20 {
		let low = load_10(bytes.as_ptr());
		let high = load_10(bytes.as_ptr().add(10));
		let input = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(low), high);
		_mm256_storeu_si256(dest as *mut __m256i, lookup(split(input), base));
		bytes = bytes.get_unchecked(20..);
		dest = dest.add(32);
	}

	if bytes.len() >= 10 {
		let input = load_10(bytes.as_ptr());
		let input = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(input), input);
		let ascii = lookup(split(input), base);
		_mm_storeu_si128(dest as *mut __m128i, _mm256_castsi256_si128(ascii));
		bytes = bytes.get_unchecked(10..);
		dest = dest.add(16);
	}

	scalar::encode(bytes, base, pad, dest)
}
