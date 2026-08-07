#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn select_row(values: __m128i, lower: __m128i, higher: __m128i, row: usize, base: &Base32) -> __m128i {
	let table = _mm_loadu_si128(base.lut.as_ptr().add(row * 16) as *const __m128i);
	let candidate = _mm_shuffle_epi8(table, lower);
	let selected = _mm_cmpeq_epi8(higher, _mm_set1_epi8(row as i8));
	_mm_or_si128(_mm_andnot_si128(selected, values), _mm_and_si128(selected, candidate))
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn lookup(input: __m128i, base: &Base32) -> Result<__m128i, crate::ErrorKind> {
	let lower = _mm_and_si128(input, _mm_set1_epi8(0x0f));
	let higher = _mm_and_si128(_mm_srli_epi16::<4>(input), _mm_set1_epi8(0x0f));
	let mut values = _mm_set1_epi8(-1);

	macro_rules! row {
		($row:literal) => {
			if base.lut_rows & (1 << $row) != 0 {
				values = select_row(values, lower, higher, $row, base);
			}
		};
	}
	row!(0);
	row!(1);
	row!(2);
	row!(3);
	row!(4);
	row!(5);
	row!(6);
	row!(7);

	// Valid values are 0..31. Invalid table entries and absent/non-ASCII
	// rows retain their high bit.
	if _mm_movemask_epi8(values) != 0 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}
	Ok(values)
}

/// Pack two groups of eight five-bit indices into ten bytes.
#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn pack(values: __m128i) -> __m128i {
	let pairs = _mm_maddubs_epi16(values, _mm_set1_epi16(0x0120));
	let words = _mm_madd_epi16(pairs, _mm_set1_epi32(0x0001_0400));

	let high = _mm_shuffle_epi8(words, _mm_setr_epi8(
		2, 1, 0, -1, -1, 10, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1,
	));
	let low = _mm_shuffle_epi8(words, _mm_setr_epi8(
		1, 0, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1,
	));
	let middle = _mm_shuffle_epi8(words, _mm_setr_epi8(
		-1, -1, 6, 5, 4, -1, -1, 14, 13, 12, -1, -1, -1, -1, -1, -1,
	));
	let high = _mm_and_si128(_mm_slli_epi16::<4>(high), _mm_set1_epi8(0xf0u8 as i8));
	let low = _mm_and_si128(_mm_srli_epi16::<4>(low), _mm_set1_epi8(0x0f));
	_mm_or_si128(_mm_or_si128(high, low), middle)
}

#[inline]
#[target_feature(enable = "ssse3")]
unsafe fn store(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	(dest.add(8) as *mut u16).write_unaligned(_mm_extract_epi16::<4>(value) as u16);
}

#[target_feature(enable = "ssse3")]
pub unsafe fn decode(mut string: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	// One vector alone does not amortize the dynamic row lookup on this ISA.
	if string.len() < 32 {
		return scalar::decode(string, base, pad, dest);
	}

	while string.len() >= 16 {
		let input = _mm_loadu_si128(string.as_ptr() as *const __m128i);
		let Ok(values) = lookup(input, base)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};
		store(pack(values), dest);
		string = string.get_unchecked(16..);
		dest = dest.add(10);
	}

	scalar::decode(string, base, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}
