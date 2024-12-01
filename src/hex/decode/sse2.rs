#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

// http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html#algorithm-3-by-geoff-langdale
#[inline]
#[target_feature(enable = "sse2")]
unsafe fn decode_hex(v: __m128i) -> Result<__m128i, crate::Error> {
	// Move digits '0'..'9' into range 0xf6..0xff.
	let t1 = _mm_add_epi8(v, _mm_set1_epi8((0xff - b'9') as i8));

	// And then correct the range to 0xf0..0xf9. All other bytes become less than 0xf0.
	let t2 = _mm_sub_epi8(t1, _mm_set1_epi8(6));

	// Convert '0'..'9' into nibbles 0..9. Non-digit bytes become greater than 0x0f.
	let t3 = _mm_sub_epi8(t2, _mm_set1_epi8(0xf0u8 as i8));

	// Convert into uppercase 'a'..'f' => 'A'..'F'.
	let t4 = _mm_and_si128(v, _mm_set1_epi8(0xdfu8 as i8));

	// Move hex letter 'A'..'F' into range 0..5.
	let t5 = _mm_sub_epi8(t4, _mm_set1_epi8(b'A' as i8));

	// And correct the range into 10..15. The non-hex letters bytes become greater than 0x0f.
	let t6 = _mm_adds_epu8(t5, _mm_set1_epi8(10));

	// Finally choose the result: either valid nibble (0..9/10..15) or some byte greater than 0x0f.
	let t7 = _mm_min_epu8(t3, t6);

	// Detect errors, i.e. bytes greater than 15. As SSE does not provide an unsigned compare, we have to use a trick with the saturated add.
	let t8 = _mm_adds_epu8(t7, _mm_set1_epi8(127-15));

	if _mm_movemask_epi8(t8) != 0 {
		return Err(crate::Error::InvalidCharacter);
	}

	Ok(t7)
}

// unsafe fn nibbles2bytes(result: __m128i) -> __m128i {
// 	let t0 = _mm_maddubs_epi16(result, _mm_set1_epi16(0x0110));
// 	let t1 = _mm_setr_epi8(
// 		14, 12, 10, 8, 6, 4, 2, 0,
// 		-1, -1, -1, -1, -1, -1, -1, -1);
// 	let t2 = _mm_shuffle_epi8(t0, t1);
// 	return t2;
// }

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn nibbles2bytes(result: __m128i) -> __m128i {
	let t3 = _mm_or_si128(
		_mm_slli_epi16(result, 4),
		_mm_bsrli_si128(result, 1));
	let t4 = _mm_and_si128(t3, _mm_set1_epi16(0x00ff));
	let t5 = _mm_packus_epi16(t4, _mm_setzero_si128());

	return t5;
}

#[target_feature(enable = "sse2")]
pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	while string.len() >= 16 {
		let v1 = _mm_loadu_si128(string.as_ptr() as *const __m128i);
		let v2 = decode_hex(v1)?;
		let v3 = nibbles2bytes(v2);
		_mm_storeu_si64(dest, v3);

		dest = dest.add(8);
		string = &string[16..];
	}

	scalar::decode(string, dest as *mut u8)
}
