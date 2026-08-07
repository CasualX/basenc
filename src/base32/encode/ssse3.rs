#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

/// Load exactly ten bytes, leaving the other six lanes initialized to zero.
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn load_10(bytes: *const u8) -> __m128i {
	let low = _mm_loadl_epi64(bytes as *const __m128i);
	_mm_insert_epi16::<4>(low, (bytes.add(8) as *const u16).read_unaligned() as i32)
}

/// Split two five-byte blocks into sixteen five-bit indices.
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn split(input: __m128i) -> __m128i {
	// Each 16-bit lane is a big-endian window beginning at the relevant
	// source bit. Multiplication-high supplies the per-lane right shift.
	let first = _mm_shuffle_epi8(input, _mm_setr_epi8(
		1, 0, 1, 0, 2, 1, 2, 1, 3, 2, 4, 3, 4, 3, 4, 4,
	));
	let second = _mm_shuffle_epi8(input, _mm_setr_epi8(
		6, 5, 6, 5, 7, 6, 7, 6, 8, 7, 9, 8, 9, 8, 9, 9,
	));
	let shifts = _mm_setr_epi16(32, 1024, 128, 4096, 512, 64, 2048, 256);
	let mask = _mm_set1_epi16(0x1f);
	let first = _mm_and_si128(_mm_mulhi_epu16(first, shifts), mask);
	let second = _mm_and_si128(_mm_mulhi_epu16(second, shifts), mask);
	_mm_packus_epi16(first, second)
}

/// Translate arbitrary indices through the caller-provided alphabet.
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(indices: __m128i, base: &Base32) -> __m128i {
	let low_charset = _mm_loadu_si128(base.charset.as_ptr() as *const __m128i);
	let high_charset = _mm_loadu_si128(base.charset.as_ptr().add(16) as *const __m128i);
	let high_mask = _mm_cmpgt_epi8(indices, _mm_set1_epi8(15));
	let low = _mm_shuffle_epi8(low_charset, indices);
	let high = _mm_shuffle_epi8(high_charset, indices);
	_mm_or_si128(_mm_andnot_si128(high_mask, low), _mm_and_si128(high_mask, high))
}

#[target_feature(enable = "ssse3")]
pub unsafe fn encode(mut bytes: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	// A full-width unaligned load is cheaper than assembling ten bytes. Only
	// consume the ten bytes represented by the output.
	while bytes.len() >= 16 {
		let input = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
		let ascii = lookup(split(input), base);
		_mm_storeu_si128(dest as *mut __m128i, ascii);
		bytes = bytes.get_unchecked(10..);
		dest = dest.add(16);
	}

	while bytes.len() >= 10 {
		let ascii = lookup(split(load_10(bytes.as_ptr())), base);
		_mm_storeu_si128(dest as *mut __m128i, ascii);
		bytes = bytes.get_unchecked(10..);
		dest = dest.add(16);
	}

	scalar::encode(bytes, base, pad, dest)
}
