// http://0x80.pl/notesen/2016-01-17-sse-base64-decoding.html

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "sse2")]
pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	if string.is_empty() {
		return Ok(dest);
	}

	while string.len() >= 16 {
		let block = _mm_loadu_si128(string.as_ptr() as *const __m128i);

		let Ok(values) = lookup(block, base)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};

		let packed = pack(values);
		let compacted = compact(packed);
		store(compacted, dest);

		dest = dest.add(12);
		string = string.get_unchecked(16..);
	}

	scalar::decode(string, base, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}

//----------------------------------------------------------------

/// Store the 12 decoded bytes without touching the four bytes past the output.
#[inline]
#[target_feature(enable = "sse2")]
unsafe fn store(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	let high = _mm_srli_si128::<8>(value);
	(dest.add(8) as *mut u32).write_unaligned(_mm_cvtsi128_si32(high) as u32);
}

//----------------------------------------------------------------

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn lookup(input: __m128i, base: &Base64) -> Result<__m128i, crate::ErrorKind> {
	#![allow(non_snake_case)]

	// shift for range 'A' - 'Z'
	let ge_A = _mm_cmpgt_epi8(input, _mm_set1_epi8(b'A' as i8 - 1));
	let le_Z = _mm_cmplt_epi8(input, _mm_set1_epi8(b'Z' as i8 + 1));
	let mask_AZ = _mm_and_si128(ge_A, le_Z);
	let shift_AZ = _mm_set1_epi8(0u8.wrapping_sub(b'A') as i8);
	let range_AZ = _mm_and_si128(shift_AZ, mask_AZ);

	// shift range for 'a' - 'z'
	let ge_a = _mm_cmpgt_epi8(input, _mm_set1_epi8(b'a' as i8 - 1));
	let le_z = _mm_cmplt_epi8(input, _mm_set1_epi8(b'z' as i8 + 1));
	let mask_az = _mm_and_si128(ge_a, le_z);
	let shift_az = _mm_set1_epi8(26u8.wrapping_sub(b'a') as i8);
	let range_az = _mm_and_si128(shift_az, mask_az);

	// shift for range '0' - '9'
	let ge_0 = _mm_cmpgt_epi8(input, _mm_set1_epi8(b'0' as i8 - 1));
	let le_9 = _mm_cmplt_epi8(input, _mm_set1_epi8(b'9' as i8 + 1));
	let mask_09 = _mm_and_si128(ge_0, le_9);
	let shift_09 = _mm_set1_epi8(52u8.wrapping_sub(b'0') as i8);
	let range_09 = _mm_and_si128(shift_09, mask_09);

	// shift for character '+'
	let eq_char62 = _mm_cmpeq_epi8(input, _mm_set1_epi8(base.charset[62] as i8));
	let shift_char62 = _mm_set1_epi8(62u8.wrapping_sub(base.charset[62]) as i8);
	let char_char62 = _mm_and_si128(shift_char62, eq_char62);

	// shift for character '/'
	let eq_char63 = _mm_cmpeq_epi8(input, _mm_set1_epi8(base.charset[63] as i8));
	let shift_char63 = _mm_set1_epi8(63u8.wrapping_sub(base.charset[63]) as i8);
	let char_char63 = _mm_and_si128(shift_char63, eq_char63);

	// merge partial results
	let shift = _mm_or_si128(range_AZ,
		_mm_or_si128(range_az,
		_mm_or_si128(range_09,
		_mm_or_si128(char_char62, char_char63))));

	// check for errors
	let valid = _mm_or_si128(mask_AZ,
		_mm_or_si128(mask_az,
		_mm_or_si128(mask_09,
		_mm_or_si128(eq_char62, eq_char63))));
	if _mm_movemask_epi8(valid) != 0xffff {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	Ok(_mm_add_epi8(input, shift))
}

// input:  [00dddddd|00cccccc|00bbbbbb|00aaaaaa]
// result: [00000000|aaaaaabb|bbbbcccc|ccdddddd]
#[inline]
#[target_feature(enable = "sse2")]
unsafe fn pack(values: __m128i) -> __m128i {
	let ca = _mm_and_si128(values, _mm_set1_epi32(0x003f003f));
	let db = _mm_and_si128(values, _mm_set1_epi32(0x3f003f00));

	// t0   =  [0000cccc|ccdddddd|0000aaaa|aabbbbbb]
	let t0 = _mm_or_si128(
		_mm_srli_epi32(db, 8),
		_mm_slli_epi32(ca, 6),
	);

	// t1   =  [dddd0000|aaaaaabb|bbbbcccc|dddddddd]
	let t1 = _mm_or_si128(
		_mm_srli_epi32(t0, 16),
		_mm_slli_epi32(t0, 12),
	);

	return _mm_and_si128(t1, _mm_set1_epi32(0x00ffffff));
}

// Compact the 24 bit words packed into 32 bit lanes
// packed: [0123|0123|0123|0123]
// result: [0000|3213|2132|1321]
#[inline]
#[target_feature(enable = "sse2")]
unsafe fn compact(packed: __m128i) -> __m128i {
	// Byte swap the high and low bytes of the 24 bit words
	let packed = {
		let a = _mm_srli_epi32(packed, 16);
		let b = _mm_slli_epi32(packed, 16);
		let c = _mm_setr_epi8(0, -1, 0, 0,  0, -1, 0, 0,  0, -1, 0, 0,  0, -1, 0, 0);
		_mm_or_si128(_mm_or_si128(a, b), _mm_and_si128(packed, c))
	};

	let mask0 = _mm_setr_epi8(-1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
	let v0 = _mm_and_si128(packed, mask0);
	let mask1 = _mm_setr_epi8(0, 0, 0, 0,  -1, -1, -1, 0,   0,  0,  0,  0,   0,  0,  0,  0);
	let v1 = _mm_bsrli_si128(_mm_and_si128(packed, mask1), 1);
	let mask2 = _mm_setr_epi8(0, 0, 0, 0,   0,  0,  0,  0,  -1, -1, -1, 0,   0,  0,  0,  0);
	let v2 = _mm_bsrli_si128(_mm_and_si128(packed, mask2), 2);
	let mask3 = _mm_setr_epi8(0, 0, 0, 0,   0,  0,  0,  0,   0,  0,  0,  0,  -1, -1, -1, 0);
	let v3 = _mm_bsrli_si128(_mm_and_si128(packed, mask3), 3);
	return _mm_or_si128(_mm_or_si128(v0, v1), _mm_or_si128(v2, v3));
}
