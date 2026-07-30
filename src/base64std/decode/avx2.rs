#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "avx2")]
pub unsafe fn decode(mut string: &[u8], pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 32 {
		let consumed = decode_blocks(string, dest);
		string = string.get_unchecked(consumed..);
		dest = dest.add(consumed / 4 * 3);
		if string.len() >= 32 {
			dest = scalar::decode_chunk(&mut string, pad, dest)?;
		}
	}

	while string.len() >= 16 {
		let block = _mm_loadu_si128(string.as_ptr() as *const __m128i);
		let lanes = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(block), block);
		let Ok(values) = lookup(lanes)
		else {
			dest = scalar::decode_chunk(&mut string, pad, dest)?;
			continue;
		};
		store_lane(_mm256_castsi256_si128(compact(pack(values))), dest);
		dest = dest.add(12);
		string = string.get_unchecked(16..);
	}

	scalar::decode(string, pad, dest)
}

#[inline(never)]
#[target_feature(enable = "avx2")]
unsafe fn decode_blocks(string: &[u8], dest: *mut u8) -> usize {
	let mut input_offset = 0;
	let mut output_offset = 0;
	while input_offset <= string.len() - 32 {
		let block = _mm256_loadu_si256(string.as_ptr().add(input_offset) as *const __m256i);
		let Ok(values) = lookup(block)
		else {
			break;
		};
		store(compact(pack(values)), dest.add(output_offset));
		input_offset += 32;
		output_offset += 24;
	}
	input_offset
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store(value: __m256i, dest: *mut u8) {
	store_lane(_mm256_castsi256_si128(value), dest);
	store_lane(_mm256_extracti128_si256::<1>(value), dest.add(12));
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store_lane(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	(dest.add(8) as *mut u32).write_unaligned(_mm_cvtsi128_si32(_mm_srli_si128::<8>(value)) as u32);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn lookup(input: __m256i) -> Result<__m256i, crate::Error> {
	let higher_nibble = _mm256_and_si256(_mm256_srli_epi32(input, 4), _mm256_set1_epi8(0x0f));
	let lower_nibble = _mm256_and_si256(input, _mm256_set1_epi8(0x0f));
	let lower_lut = _mm256_setr_epi8(
		0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
		0x11, 0x11, 0x13, 0x1a, 0x1b, 0x1b, 0x1b, 0x1a,
		0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
		0x11, 0x11, 0x13, 0x1a, 0x1b, 0x1b, 0x1b, 0x1a,
	);
	let higher_lut = _mm256_setr_epi8(
		0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08,
		0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
		0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08,
		0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
	);
	let invalid = _mm256_and_si256(
		_mm256_shuffle_epi8(lower_lut, lower_nibble),
		_mm256_shuffle_epi8(higher_lut, higher_nibble),
	);
	if _mm256_testz_si256(invalid, invalid) == 0 {
		return Err(crate::Error::InvalidCharacter);
	}

	let slash = _mm256_cmpeq_epi8(input, _mm256_set1_epi8(b'/' as i8));
	let offset_index = _mm256_add_epi8(higher_nibble, slash);
	let offset_lut = _mm256_setr_epi8(
		0, 16, 19, 4, -65, -65, -71, -71,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 16, 19, 4, -65, -65, -71, -71,
		0, 0, 0, 0, 0, 0, 0, 0,
	);
	Ok(_mm256_add_epi8(input, _mm256_shuffle_epi8(offset_lut, offset_index)))
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
	_mm256_shuffle_epi8(packed, _mm256_setr_epi8(
		2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
		2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
	))
}
