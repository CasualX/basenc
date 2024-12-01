// http://0x80.pl/notesen/2016-01-12-sse-base64-encoding.html

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn split_bytes(value: __m128i) -> __m128i {
	let input = _mm_shuffle_epi8(value, _mm_setr_epi8(
		1, 0, 2, 1,
		4, 3, 5, 4,
		7, 6, 8, 7,
		10, 9, 11, 10,
	));

	// t0 = [0000cccc|CC000000|aaaaaa00|00000000]
	let t0 = _mm_and_si128(input, _mm_set1_epi32(0x0fc0fc00));

	// t1    = [00000000|00cccccc|00000000|00aaaaaa]
	let t1 = _mm_mulhi_epu16(t0, _mm_set1_epi32(0x04000040));

	// t2    = [00000000|00dddddd|000000bb|bbbb0000]
	let t2 = _mm_and_si128(input, _mm_set1_epi32(0x003f03f0));

	// t3    = [00dddddd|00000000|00bbbbbb|00000000](
	let t3 = _mm_mullo_epi16(t2, _mm_set1_epi32(0x01000010));

	// res   = [00dddddd|00cccccc|00bbbbbb|00aaaaaa] = t1 | t3
	let indices = _mm_or_si128(t1, t3);

	return indices;
}

/* Naive implementation
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(i: __m128i, base: &Base64) -> __m128i {
	#![allow(non_snake_case)]

	let less_26 = _mm_cmplt_epi8(i, _mm_set1_epi8(26));
	let less_52 = _mm_cmplt_epi8(i, _mm_set1_epi8(52));
	let less_62 = _mm_cmplt_epi8(i, _mm_set1_epi8(62));
	let equal_62 = _mm_cmpeq_epi8(i, _mm_set1_epi8(62));
	let equal_63 = _mm_cmpeq_epi8(i, _mm_set1_epi8(63));

	let range_AZ = _mm_and_si128(_mm_set1_epi8(b'A' as i8), less_26);
	let range_az = _mm_and_si128(_mm_set1_epi8(b'a' as i8 - 26), _mm_andnot_si128(less_26, less_52));
	let range_09 = _mm_and_si128(_mm_set1_epi8(b'0' as i8 - 52), _mm_andnot_si128(less_52, less_62));
	let range_plus = _mm_and_si128(_mm_set1_epi8(base.charset[62].wrapping_sub(62) as i8), equal_62);
	let range_slash = _mm_and_si128(_mm_set1_epi8(base.charset[63].wrapping_sub(63) as i8), equal_63);

	let shift = _mm_or_si128(_mm_or_si128(_mm_or_si128(_mm_or_si128(range_AZ, range_az), range_09), range_plus), range_slash);

	return _mm_add_epi8(i, shift);
}
*/

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(input: __m128i, base: &Base64) -> __m128i {
	// reduce  0..51 -> 0
	//        52..61 -> 1 .. 10
	//            62 -> 11
	//            63 -> 12
	let result = _mm_subs_epu8(input, _mm_set1_epi8(51));

	// distinguish between ranges 0..25 and 26..51:
	//         0 .. 25 -> remains 0
	//        26 .. 51 -> becomes 13
	let less = _mm_cmpgt_epi8(_mm_set1_epi8(26), input);
	let result = _mm_or_si128(result, _mm_and_si128(less, _mm_set1_epi8(13)));

	let _a = b'a' as i8 - 26;
	let _0 = b'0' as i8 - 52;
	let shift_lut = _mm_setr_epi8(
		_a, _0, _0, _0, _0, _0,
		_0, _0, _0, _0, _0, base.charset[62].wrapping_sub(62) as i8,
		base.charset[63].wrapping_sub(63) as i8, b'A' as i8, 0, 0,
	);

	// read shift
	let result = _mm_shuffle_epi8(shift_lut, result);

	return _mm_add_epi8(result, input);
}

#[target_feature(enable = "ssse3")]
pub unsafe fn encode(mut bytes: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> *mut u8 {
	while bytes.len() >= 16 {
		let data = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
		let split = split_bytes(data);
		let ascii = lookup(split, base);
		_mm_storeu_si128(dest as *mut __m128i, ascii);

		bytes = &bytes[12..];
		dest = dest.offset(16);
	}

	scalar::encode(bytes, base, pad, dest)
}
