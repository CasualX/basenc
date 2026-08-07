#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "ssse3")]
pub unsafe fn decode(mut string: &[u8], pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 16 {
		let block = _mm_loadu_si128(string.as_ptr() as *const __m128i);
		let Ok(values) = lookup(block)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};
		store(compact(pack(values)), dest);
		dest = dest.add(12);
		string = string.get_unchecked(16..);
	}
	scalar::decode(string, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn store(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	(dest.add(8) as *mut u32).write_unaligned(_mm_cvtsi128_si32(_mm_srli_si128::<8>(value)) as u32);
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(input: __m128i) -> Result<__m128i, crate::ErrorKind> {
	let higher_nibble = _mm_and_si128(_mm_srli_epi32(input, 4), _mm_set1_epi8(0x0f));
	let lower_nibble = _mm_and_si128(input, _mm_set1_epi8(0x0f));
	let lower_lut = _mm_setr_epi8(
		0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
		0x11, 0x11, 0x13, 0x1a, 0x1b, 0x1b, 0x1b, 0x1a,
	);
	let higher_lut = _mm_setr_epi8(
		0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08,
		0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
	);
	let invalid = _mm_and_si128(
		_mm_shuffle_epi8(lower_lut, lower_nibble),
		_mm_shuffle_epi8(higher_lut, higher_nibble),
	);
	if _mm_movemask_epi8(_mm_cmpeq_epi8(invalid, _mm_setzero_si128())) != 0xffff {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let slash = _mm_cmpeq_epi8(input, _mm_set1_epi8(b'/' as i8));
	let offset_index = _mm_add_epi8(higher_nibble, slash);
	let offset_lut = _mm_setr_epi8(
		0, 16, 19, 4, -65, -65, -71, -71,
		0, 0, 0, 0, 0, 0, 0, 0,
	);
	Ok(_mm_add_epi8(input, _mm_shuffle_epi8(offset_lut, offset_index)))
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
