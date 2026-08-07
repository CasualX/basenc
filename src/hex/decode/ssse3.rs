#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

// http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html#algorithm-3-by-geoff-langdale
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn decode_hex(v: __m128i) -> __m128i {
	let digit = _mm_sub_epi8(_mm_subs_epu8(_mm_add_epi8(v, _mm_set1_epi8((0xff - b'9') as i8)), _mm_set1_epi8(6)), _mm_set1_epi8(0xf0u8 as i8));
	let letter = _mm_adds_epu8(_mm_sub_epi8(_mm_and_si128(v, _mm_set1_epi8(0xdfu8 as i8)), _mm_set1_epi8(b'A' as i8)), _mm_set1_epi8(10));
	_mm_min_epu8(digit, letter)
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn validate_hex(result: __m128i) -> Result<(), crate::ErrorKind> {
	let checked = _mm_adds_epu8(result, _mm_set1_epi8(127 - 15));

	if _mm_movemask_epi8(checked) != 0 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	Ok(())
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn nibbles2bytes(nibbles: __m128i) -> __m128i {
	let pairs = _mm_maddubs_epi16(nibbles, _mm_set1_epi16(0x0110));
	_mm_shuffle_epi8(pairs, _mm_setr_epi8(0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1))
}

#[target_feature(enable = "ssse3")]
pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 32 {
		let src = string.as_ptr() as *const __m128i;
		let a = decode_hex(_mm_loadu_si128(src));
		let b = decode_hex(_mm_loadu_si128(src.add(1)));
		if validate_hex(_mm_max_epu8(a, b)).is_err() {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		}
		let bytes = _mm_unpacklo_epi64(nibbles2bytes(a), nibbles2bytes(b));
		_mm_storeu_si128(dest as *mut __m128i, bytes);

		dest = dest.add(16);
		string = &string[32..];
	}

	if string.len() >= 16 {
		let nibbles = decode_hex(_mm_loadu_si128(string.as_ptr() as *const __m128i));
		if validate_hex(nibbles).is_err() {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		}
		let bytes = nibbles2bytes(nibbles);
		_mm_storeu_si64(dest, bytes);

		dest = dest.add(8);
		string = &string[16..];
	}

	scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()))
}
