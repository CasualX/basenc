// http://0x80.pl/notesen/2016-01-17-sse-base64-decoding.html

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "ssse3")]
pub unsafe fn decode(mut string: &[u8], base: &Base64, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	if string.len() == 0 {
		return Ok(dest);
	}

	while string.len() >= 16 {
		let block = _mm_loadu_si128(string.as_ptr() as *const __m128i);

		let Ok(values) = lookup(block, base)
		else {
			// Handle errors and padding with the scalar code path
			dest = scalar::decode(&string[..16], base, pad, dest)?;
			string = &string[16..];
			continue;
		};

		let packed = pack(values);
		let compacted = compact(packed);
		let mov_mask = _mm_setr_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0);
		_mm_maskmoveu_si128(compacted, mov_mask, dest as *mut i8);

		dest = dest.offset(12);
		string = &string[16..];
		continue;
	}

	scalar::decode(string, base, pad, dest)
}

//----------------------------------------------------------------

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(input: __m128i, base: &Base64) -> Result<__m128i, crate::Error> {
	let higher_nibble = _mm_and_si128(_mm_srli_epi32(input, 4), _mm_set1_epi8(0x0f));

	let linv = 1;
	let hinv = 0;

	let lower_bound_lut = _mm_setr_epi8(
		/* 0 */ linv, /* 1 */ linv, /* 2 */ linv, /* 3 */ 0x30,
		/* 4 */ 0x41, /* 5 */ 0x50, /* 6 */ 0x61, /* 7 */ 0x70,
		/* 8 */ linv, /* 9 */ linv, /* a */ linv, /* b */ linv,
		/* c */ linv, /* d */ linv, /* e */ linv, /* f */ linv,
	);
	let upper_bound_lut = _mm_setr_epi8(
		/* 0 */ hinv, /* 1 */ hinv, /* 2 */ hinv, /* 3 */ 0x39,
		/* 4 */ 0x4f, /* 5 */ 0x5a, /* 6 */ 0x6f, /* 7 */ 0x7a,
		/* 8 */ hinv, /* 9 */ hinv, /* a */ hinv, /* b */ hinv,
		/* c */ hinv, /* d */ hinv, /* e */ hinv, /* f */ hinv,
	);

	// the difference between the shift and lower bound
	let shuft_lut = _mm_setr_epi8(
		/* 0 */ 0x00,        /* 1 */ 0x00,        /* 2 */ 0x00,        /* 3 */ 0x34 - 0x30,
		/* 4 */ 0x00 - 0x41, /* 5 */ 0x0f - 0x50, /* 6 */ 0x1a - 0x61, /* 7 */ 0x29 - 0x70,
		/* 8 */ 0x00,        /* 9 */ 0x00,        /* a */ 0x00,        /* b */ 0x00,
		/* c */ 0x00,        /* d */ 0x00,        /* e */ 0x00,        /* f */ 0x00
	);

	let upper_bound = _mm_shuffle_epi8(upper_bound_lut, higher_nibble);
	let lower_bound = _mm_shuffle_epi8(lower_bound_lut, higher_nibble);

	let mask_below = _mm_cmplt_epi8(input, lower_bound);
	let mask_above = _mm_cmpgt_epi8(input, upper_bound);
	let mask_eq_62 = _mm_cmpeq_epi8(input, _mm_set1_epi8(base.charset[62] as i8));
	let mask_eq_63 = _mm_cmpeq_epi8(input, _mm_set1_epi8(base.charset[63] as i8));
	let mask_eq = _mm_or_si128(mask_eq_62, mask_eq_63);

	let outside = _mm_andnot_si128(mask_eq, _mm_or_si128(mask_below, mask_above));
	if _mm_movemask_epi8(outside) != 0 {
		return Err(crate::Error::InvalidCharacter);
	}

	let shift_bound = _mm_shuffle_epi8(shuft_lut, higher_nibble);
	let shift_eq = _mm_or_si128(
		_mm_and_si128(mask_eq_62, _mm_set1_epi8(62u8.wrapping_sub(base.charset[62]) as i8)),
		_mm_and_si128(mask_eq_63, _mm_set1_epi8(63u8.wrapping_sub(base.charset[63]) as i8)));
	let shift = _mm_or_si128(_mm_andnot_si128(mask_eq, shift_bound), shift_eq);

	let result = _mm_add_epi8(input, shift);

	Ok(result)
}

// input:  [00dddddd|00cccccc|00bbbbbb|00aaaaaa]
// result: [00000000|aaaaaabb|bbbbcccc|ccdddddd]
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn pack(values: __m128i) -> __m128i {
	// merge:  [0000cccc|ccdddddd|0000aaaa|aabbbbbb]
	let merge_ab_and_bc = _mm_maddubs_epi16(values, _mm_set1_epi32(0x01400140));
	return _mm_madd_epi16(merge_ab_and_bc, _mm_set1_epi32(0x00011000));
}

// Compact the 24 bit words packed into 32 bit lanes
// packed: [0123|0123|0123|0123]
// result: [0000|3213|2132|1321]
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn compact(packed: __m128i) -> __m128i {
	let mask = _mm_setr_epi8(2, 1, 0,  6, 5, 4,  10, 9, 8,  14, 13, 12,  -1, -1, -1, -1);
	return _mm_shuffle_epi8(packed, mask);
}
