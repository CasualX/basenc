#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn select_row(values: __m256i, lower: __m256i, higher: __m256i, row: usize, base: &Base32) -> __m256i {
	let table = _mm_loadu_si128(base.lut.as_ptr().add(row * 16) as *const __m128i);
	let table = _mm256_broadcastsi128_si256(table);
	let candidate = _mm256_shuffle_epi8(table, lower);
	let selected = _mm256_cmpeq_epi8(higher, _mm256_set1_epi8(row as i8));
	_mm256_or_si256(_mm256_andnot_si256(selected, values), _mm256_and_si256(selected, candidate))
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn lookup(input: __m256i, base: &Base32) -> Result<__m256i, crate::ErrorKind> {
	let lower = _mm256_and_si256(input, _mm256_set1_epi8(0x0f));
	let higher = _mm256_and_si256(_mm256_srli_epi16::<4>(input), _mm256_set1_epi8(0x0f));
	let mut values = _mm256_set1_epi8(-1);

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

	if _mm256_movemask_epi8(values) != 0 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}
	Ok(values)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn pack(values: __m256i) -> __m256i {
	let pairs = _mm256_maddubs_epi16(values, _mm256_set1_epi16(0x0120));
	let words = _mm256_madd_epi16(pairs, _mm256_set1_epi32(0x0001_0400));

	let high = _mm256_shuffle_epi8(words, _mm256_setr_epi8(
		2, 1, 0, -1, -1, 10, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1,
		2, 1, 0, -1, -1, 10, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1,
	));
	let low = _mm256_shuffle_epi8(words, _mm256_setr_epi8(
		1, 0, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1,
		1, 0, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1,
	));
	let middle = _mm256_shuffle_epi8(words, _mm256_setr_epi8(
		-1, -1, 6, 5, 4, -1, -1, 14, 13, 12, -1, -1, -1, -1, -1, -1,
		-1, -1, 6, 5, 4, -1, -1, 14, 13, 12, -1, -1, -1, -1, -1, -1,
	));
	let high = _mm256_and_si256(_mm256_slli_epi16::<4>(high), _mm256_set1_epi8(0xf0u8 as i8));
	let low = _mm256_and_si256(_mm256_srli_epi16::<4>(low), _mm256_set1_epi8(0x0f));
	_mm256_or_si256(_mm256_or_si256(high, low), middle)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store_lane(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	(dest.add(8) as *mut u16).write_unaligned(_mm_extract_epi16::<4>(value) as u16);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store(value: __m256i, dest: *mut u8) {
	store_lane(_mm256_castsi256_si128(value), dest);
	store_lane(_mm256_extracti128_si256::<1>(value), dest.add(10));
}

#[target_feature(enable = "avx2")]
pub unsafe fn decode(mut string: &[u8], base: &Base32, pad: Padding, mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 32 {
		let input = _mm256_loadu_si256(string.as_ptr() as *const __m256i);
		let Ok(values) = lookup(input, base)
		else {
			let offset = input_len - string.len();
			dest = scalar::decode_chunk(&mut string, base, pad, dest).map_err(|error| error.shifted(offset))?;
			continue;
		};
		store(pack(values), dest);
		string = string.get_unchecked(32..);
		dest = dest.add(20);
	}

	if string.len() >= 16 {
		let input = _mm_loadu_si128(string.as_ptr() as *const __m128i);
		let input = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(input), input);
		let Ok(values) = lookup(input, base)
		else {
			return scalar::decode(string, base, pad, dest).map_err(|error| error.shifted(input_len - string.len()));
		};
		store_lane(_mm256_castsi256_si128(pack(values)), dest);
		string = string.get_unchecked(16..);
		dest = dest.add(10);
	}

	scalar::decode(string, base, pad, dest).map_err(|error| error.shifted(input_len - string.len()))
}
