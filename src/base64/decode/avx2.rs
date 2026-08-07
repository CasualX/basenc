#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "avx2")]
pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 32 {
		let block = _mm256_loadu_si256(string.as_ptr() as *const __m256i);
		let Ok(values) = lookup(block, base)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};

		let packed = pack(values);
		let compacted = compact(packed);
		store(compacted, dest);

		dest = dest.add(24);
		string = string.get_unchecked(32..);
	}

	// Preserve the 128-bit path for medium-sized inputs and AVX2 tails.
	while string.len() >= 16 {
		let block = _mm_loadu_si128(string.as_ptr() as *const __m128i);
		let lanes = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(block), block);
		let Ok(values) = lookup(lanes, base)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};

		let compacted = compact(pack(values));
		store_lane(_mm256_castsi256_si128(compacted), dest);
		dest = dest.add(12);
		string = string.get_unchecked(16..);
	}

	scalar::decode(string, base, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store(value: __m256i, dest: *mut u8) {
	let low = _mm256_castsi256_si128(value);
	let high = _mm256_extracti128_si256::<1>(value);
	store_lane(low, dest);
	store_lane(high, dest.add(12));
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store_lane(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	let high = _mm_srli_si128::<8>(value);
	(dest.add(8) as *mut u32).write_unaligned(_mm_cvtsi128_si32(high) as u32);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn lookup(input: __m256i, base: &Base64) -> Result<__m256i, crate::ErrorKind> {
	let higher_nibble = _mm256_and_si256(_mm256_srli_epi32(input, 4), _mm256_set1_epi8(0x0f));

	let invalid_lower = 1;
	let invalid_upper = 0;
	let lower_bound_lut = _mm256_setr_epi8(
		invalid_lower, invalid_lower, invalid_lower, 0x30, 0x41, 0x50, 0x61, 0x70,
		invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower,
		invalid_lower, invalid_lower, invalid_lower, 0x30, 0x41, 0x50, 0x61, 0x70,
		invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower, invalid_lower,
	);
	let upper_bound_lut = _mm256_setr_epi8(
		invalid_upper, invalid_upper, invalid_upper, 0x39, 0x4f, 0x5a, 0x6f, 0x7a,
		invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper,
		invalid_upper, invalid_upper, invalid_upper, 0x39, 0x4f, 0x5a, 0x6f, 0x7a,
		invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper, invalid_upper,
	);
	let shift_lut = _mm256_setr_epi8(
		0, 0, 0, 0x34 - 0x30, 0x00 - 0x41, 0x0f - 0x50, 0x1a - 0x61, 0x29 - 0x70,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0x34 - 0x30, 0x00 - 0x41, 0x0f - 0x50, 0x1a - 0x61, 0x29 - 0x70,
		0, 0, 0, 0, 0, 0, 0, 0,
	);

	let upper_bound = _mm256_shuffle_epi8(upper_bound_lut, higher_nibble);
	let lower_bound = _mm256_shuffle_epi8(lower_bound_lut, higher_nibble);
	let below = _mm256_cmpgt_epi8(lower_bound, input);
	let above = _mm256_cmpgt_epi8(input, upper_bound);
	let equal62 = _mm256_cmpeq_epi8(input, _mm256_set1_epi8(base.charset[62] as i8));
	let equal63 = _mm256_cmpeq_epi8(input, _mm256_set1_epi8(base.charset[63] as i8));
	let equal = _mm256_or_si256(equal62, equal63);
	let outside = _mm256_andnot_si256(equal, _mm256_or_si256(below, above));
	if _mm256_movemask_epi8(outside) != 0 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let shift_bound = _mm256_shuffle_epi8(shift_lut, higher_nibble);
	let shift_equal = _mm256_or_si256(
		_mm256_and_si256(equal62, _mm256_set1_epi8(62u8.wrapping_sub(base.charset[62]) as i8)),
		_mm256_and_si256(equal63, _mm256_set1_epi8(63u8.wrapping_sub(base.charset[63]) as i8)),
	);
	let shift = _mm256_or_si256(_mm256_andnot_si256(equal, shift_bound), shift_equal);

	Ok(_mm256_add_epi8(input, shift))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn pack(values: __m256i) -> __m256i {
	let merged = _mm256_maddubs_epi16(values, _mm256_set1_epi32(0x01400140));
	_mm256_madd_epi16(merged, _mm256_set1_epi32(0x00011000))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn compact(packed: __m256i) -> __m256i {
	let mask = _mm256_setr_epi8(
		2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
		2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
	);
	_mm256_shuffle_epi8(packed, mask)
}
