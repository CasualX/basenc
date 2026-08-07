#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "sse2")]
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
#[target_feature(enable = "sse2")]
unsafe fn store(value: __m128i, dest: *mut u8) {
	_mm_storel_epi64(dest as *mut __m128i, value);
	(dest.add(8) as *mut u32).write_unaligned(_mm_cvtsi128_si32(_mm_srli_si128::<8>(value)) as u32);
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn lookup(input: __m128i) -> Result<__m128i, crate::ErrorKind> {
	let above_upper_a = _mm_cmpgt_epi8(input, _mm_set1_epi8(b'A' as i8 - 1));
	let below_upper_z = _mm_cmplt_epi8(input, _mm_set1_epi8(b'Z' as i8 + 1));
	let uppercase = _mm_and_si128(above_upper_a, below_upper_z);

	let above_lower_a = _mm_cmpgt_epi8(input, _mm_set1_epi8(b'a' as i8 - 1));
	let below_lower_z = _mm_cmplt_epi8(input, _mm_set1_epi8(b'z' as i8 + 1));
	let lowercase = _mm_and_si128(above_lower_a, below_lower_z);

	let above_zero = _mm_cmpgt_epi8(input, _mm_set1_epi8(b'0' as i8 - 1));
	let below_nine = _mm_cmplt_epi8(input, _mm_set1_epi8(b'9' as i8 + 1));
	let digit = _mm_and_si128(above_zero, below_nine);
	let plus = _mm_cmpeq_epi8(input, _mm_set1_epi8(b'+' as i8));
	let slash = _mm_cmpeq_epi8(input, _mm_set1_epi8(b'/' as i8));

	let valid = _mm_or_si128(
		uppercase,
		_mm_or_si128(lowercase, _mm_or_si128(digit, _mm_or_si128(plus, slash))),
	);
	if _mm_movemask_epi8(valid) != 0xffff {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	let shift = _mm_or_si128(
		_mm_and_si128(uppercase, _mm_set1_epi8(-(b'A' as i8))),
		_mm_or_si128(
			_mm_and_si128(lowercase, _mm_set1_epi8(26 - b'a' as i8)),
			_mm_or_si128(
				_mm_and_si128(digit, _mm_set1_epi8(52 - b'0' as i8)),
				_mm_or_si128(
					_mm_and_si128(plus, _mm_set1_epi8(62 - b'+' as i8)),
					_mm_and_si128(slash, _mm_set1_epi8(63 - b'/' as i8)),
				),
			),
		),
	);
	Ok(_mm_add_epi8(input, shift))
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn pack(values: __m128i) -> __m128i {
	let ca = _mm_and_si128(values, _mm_set1_epi32(0x003f003f));
	let db = _mm_and_si128(values, _mm_set1_epi32(0x3f003f00));
	let merged = _mm_or_si128(_mm_srli_epi32(db, 8), _mm_slli_epi32(ca, 6));
	let packed = _mm_or_si128(_mm_srli_epi32(merged, 16), _mm_slli_epi32(merged, 12));
	_mm_and_si128(packed, _mm_set1_epi32(0x00ffffff))
}

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn compact(packed: __m128i) -> __m128i {
	let swapped = {
		let high = _mm_srli_epi32(packed, 16);
		let low = _mm_slli_epi32(packed, 16);
		let middle = _mm_setr_epi8(0, -1, 0, 0, 0, -1, 0, 0, 0, -1, 0, 0, 0, -1, 0, 0);
		_mm_or_si128(_mm_or_si128(high, low), _mm_and_si128(packed, middle))
	};
	let mask0 = _mm_setr_epi8(-1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
	let mask1 = _mm_setr_epi8(0, 0, 0, 0, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0);
	let mask2 = _mm_setr_epi8(0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, 0, 0, 0, 0, 0);
	let mask3 = _mm_setr_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, 0);
	let value0 = _mm_and_si128(swapped, mask0);
	let value1 = _mm_bsrli_si128(_mm_and_si128(swapped, mask1), 1);
	let value2 = _mm_bsrli_si128(_mm_and_si128(swapped, mask2), 2);
	let value3 = _mm_bsrli_si128(_mm_and_si128(swapped, mask3), 3);
	_mm_or_si128(_mm_or_si128(value0, value1), _mm_or_si128(value2, value3))
}
