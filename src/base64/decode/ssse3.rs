// http://0x80.pl/notesen/2016-01-17-sse-base64-decoding.html

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "ssse3")]
pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 16 {
		let block = _mm_loadu_si128(string.as_ptr() as *const __m128i);
		let Ok(values) = lookup(block, base)
		else {
			dest = scalar::decode_chunk(&mut string, base, pad, dest)?;
			continue;
		};
		store(compact(pack(values)), dest);
		dest = dest.add(12);
		string = string.get_unchecked(16..);
	}
	scalar::decode(string, base, pad, dest)
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn store(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	(dest.add(8) as *mut u32).write_unaligned(_mm_cvtsi128_si32(_mm_srli_si128::<8>(value)) as u32);
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(input: __m128i, base: &Base64) -> Result<__m128i, crate::Error> {
	let higher_nibble = _mm_and_si128(_mm_srli_epi32(input, 4), _mm_set1_epi8(0x0f));
	let invalid_lower = 1;
	let invalid_upper = 0;
	let lower_bound_lut = _mm_setr_epi8(
		invalid_lower, invalid_lower, invalid_lower, 0x30, 0x41, 0x50, 0x61, 0x70,
		invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower,
	);
	let upper_bound_lut = _mm_setr_epi8(
		invalid_upper, invalid_upper, invalid_upper, 0x39, 0x4f, 0x5a, 0x6f, 0x7a,
		invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper,
	);
	let shift_lut = _mm_setr_epi8(
		0, 0, 0, 0x34 - 0x30, 0x00 - 0x41, 0x0f - 0x50, 0x1a - 0x61, 0x29 - 0x70,
		0, 0, 0, 0, 0, 0, 0, 0,
	);

	let upper_bound = _mm_shuffle_epi8(upper_bound_lut, higher_nibble);
	let lower_bound = _mm_shuffle_epi8(lower_bound_lut, higher_nibble);
	let below = _mm_cmplt_epi8(input, lower_bound);
	let above = _mm_cmpgt_epi8(input, upper_bound);
	let equal62 = _mm_cmpeq_epi8(input, _mm_set1_epi8(base.charset[62] as i8));
	let equal63 = _mm_cmpeq_epi8(input, _mm_set1_epi8(base.charset[63] as i8));
	let equal = _mm_or_si128(equal62, equal63);
	let outside = _mm_andnot_si128(equal, _mm_or_si128(below, above));
	if _mm_movemask_epi8(outside) != 0 {
		return Err(crate::Error::InvalidCharacter);
	}

	let shift_bound = _mm_shuffle_epi8(shift_lut, higher_nibble);
	let shift_equal = _mm_or_si128(
		_mm_and_si128(equal62, _mm_set1_epi8(62u8.wrapping_sub(base.charset[62]) as i8)),
		_mm_and_si128(equal63, _mm_set1_epi8(63u8.wrapping_sub(base.charset[63]) as i8)),
	);
	let shift = _mm_or_si128(_mm_andnot_si128(equal, shift_bound), shift_equal);
	Ok(_mm_add_epi8(input, shift))
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn pack(values: __m128i) -> __m128i {
	let merged = _mm_maddubs_epi16(values, _mm_set1_epi32(0x01400140));
	_mm_madd_epi16(merged, _mm_set1_epi32(0x00011000))
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn compact(packed: __m128i) -> __m128i {
	_mm_shuffle_epi8(packed, _mm_setr_epi8(
		2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
	))
}
