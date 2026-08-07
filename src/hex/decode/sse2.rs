#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

// http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html#algorithm-3-by-geoff-langdale
#[inline]
#[target_feature(enable = "sse2")]
unsafe fn decode_hex(v: __m128i) -> __m128i {
	// Move digits '0'..'9' into range 0xf6..0xff.
	let t1 = _mm_add_epi8(v, _mm_set1_epi8((0xff - b'9') as i8));

	// And then correct the range to 0xf0..0xf9. All other bytes become less than 0xf0.
	let t2 = _mm_subs_epu8(t1, _mm_set1_epi8(6));

	// Convert '0'..'9' into nibbles 0..9. Non-digit bytes become greater than 0x0f.
	let t3 = _mm_sub_epi8(t2, _mm_set1_epi8(0xf0u8 as i8));

	// Convert into uppercase 'a'..'f' => 'A'..'F'.
	let t4 = _mm_and_si128(v, _mm_set1_epi8(0xdfu8 as i8));

	// Move hex letter 'A'..'F' into range 0..5.
	let t5 = _mm_sub_epi8(t4, _mm_set1_epi8(b'A' as i8));

	// And correct the range into 10..15. The non-hex letters bytes become greater than 0x0f.
	let t6 = _mm_adds_epu8(t5, _mm_set1_epi8(10));

	// Finally choose the result: either valid nibble (0..9/10..15) or some byte greater than 0x0f.
	_mm_min_epu8(t3, t6)
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn validate_hex(result: __m128i) -> Result<(), crate::Error> {
	// Detect errors, i.e. bytes greater than 15. As SSE does not provide an unsigned compare, we have to use a trick with the saturated add.
	let checked = _mm_adds_epu8(result, _mm_set1_epi8(127 - 15));

	if _mm_movemask_epi8(checked) != 0 {
		return Err(crate::Error::InvalidCharacter);
	}

	Ok(())
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn nibbles2bytes(result: __m128i) -> __m128i {
	let hi = _mm_and_si128(_mm_slli_epi16(result, 4), _mm_set1_epi16(0x00f0));
	let lo = _mm_srli_epi16(result, 8);
	let bytes = _mm_or_si128(hi, lo);
	_mm_packus_epi16(bytes, _mm_setzero_si128())
}

#[target_feature(enable = "sse2")]
pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 32 {
		let src = string.as_ptr() as *const __m128i;
		let a = decode_hex(_mm_loadu_si128(src));
		let b = decode_hex(_mm_loadu_si128(src.add(1)));
		validate_hex(_mm_max_epu8(a, b))?;
		let bytes = _mm_unpacklo_epi64(nibbles2bytes(a), nibbles2bytes(b));
		_mm_storeu_si128(dest as *mut __m128i, bytes);

		dest = dest.add(16);
		string = &string[32..];
	}

	if string.len() >= 16 {
		let nibbles = decode_hex(_mm_loadu_si128(string.as_ptr() as *const __m128i));
		validate_hex(nibbles)?;
		_mm_storeu_si64(dest, nibbles2bytes(nibbles));

		dest = dest.add(8);
		string = &string[16..];
	}

	scalar::decode(string, dest)
}
