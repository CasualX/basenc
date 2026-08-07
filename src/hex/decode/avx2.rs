#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

// http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html#algorithm-3-by-geoff-langdale
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn decode_hex(v: __m256i) -> __m256i {
	let digit = _mm256_sub_epi8(
		_mm256_subs_epu8(
			_mm256_add_epi8(v, _mm256_set1_epi8((0xff - b'9') as i8)),
			_mm256_set1_epi8(6),
		),
		_mm256_set1_epi8(0xf0u8 as i8),
	);
	let letter = _mm256_adds_epu8(
		_mm256_sub_epi8(
			_mm256_and_si256(v, _mm256_set1_epi8(0xdfu8 as i8)),
			_mm256_set1_epi8(b'A' as i8),
		),
		_mm256_set1_epi8(10),
	);
	_mm256_min_epu8(digit, letter)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn validate_hex(result: __m256i) -> Result<(), crate::ErrorKind> {
	let invalid = _mm256_subs_epu8(result, _mm256_set1_epi8(15));

	if _mm256_testz_si256(invalid, invalid) == 0 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	Ok(())
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn decode_hex_16(v: __m128i) -> Result<__m128i, crate::ErrorKind> {
	let digit = _mm_sub_epi8(
		_mm_subs_epu8(
			_mm_add_epi8(v, _mm_set1_epi8((0xff - b'9') as i8)),
			_mm_set1_epi8(6),
		),
		_mm_set1_epi8(0xf0u8 as i8),
	);
	let letter = _mm_adds_epu8(
		_mm_sub_epi8(
			_mm_and_si128(v, _mm_set1_epi8(0xdfu8 as i8)),
			_mm_set1_epi8(b'A' as i8),
		),
		_mm_set1_epi8(10),
	);
	let result = _mm_min_epu8(digit, letter);
	let invalid = _mm_subs_epu8(result, _mm_set1_epi8(15));

	if _mm_testz_si128(invalid, invalid) == 0 {
		return Err(crate::ErrorKind::InvalidCharacter);
	}

	Ok(result)
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn nibbles2bytes(a: __m256i, b: __m256i) -> __m256i {
	let weights = _mm256_set1_epi16(0x0110);
	let a = _mm256_maddubs_epi16(a, weights);
	let b = _mm256_maddubs_epi16(b, weights);
	let packed = _mm256_packus_epi16(a, b);
	_mm256_permute4x64_epi64::<0b11_01_10_00>(packed)
}

#[target_feature(enable = "avx2")]
pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let input_len = string.len();
	while string.len() >= 64 {
		let src = string.as_ptr() as *const __m256i;
		let a = decode_hex(_mm256_loadu_si256(src));
		let b = decode_hex(_mm256_loadu_si256(src.add(1)));
		if validate_hex(_mm256_max_epu8(a, b)).is_err() {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		}
		let bytes = nibbles2bytes(a, b);
		_mm256_storeu_si256(dest as *mut __m256i, bytes);

		string = &string[64..];
		dest = dest.add(32);
	}

	if string.len() >= 32 {
		let nibbles = decode_hex(_mm256_loadu_si256(string.as_ptr() as *const __m256i));
		if validate_hex(nibbles).is_err() {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		}
		let pairs = _mm256_maddubs_epi16(nibbles, _mm256_set1_epi16(0x0110));
		let bytes = _mm256_shuffle_epi8(
			pairs,
			_mm256_setr_epi8(
				0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1,
				0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1,
			),
		);
		let bytes = _mm256_permute4x64_epi64::<0b11_01_10_00>(bytes);
		_mm_storeu_si128(dest as *mut __m128i, _mm256_castsi256_si128(bytes));

		string = &string[32..];
		dest = dest.add(16);
	}

	if string.len() >= 16 {
		let Ok(nibbles) = decode_hex_16(_mm_loadu_si128(string.as_ptr() as *const __m128i))
		else {
			return scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()));
		};
		let pairs = _mm_maddubs_epi16(nibbles, _mm_set1_epi16(0x0110));
		let bytes = _mm_shuffle_epi8(
			pairs,
			_mm_setr_epi8(0, 2, 4, 6, 8, 10, 12, 14, -1, -1, -1, -1, -1, -1, -1, -1),
		);
		_mm_storeu_si64(dest, bytes);

		string = &string[16..];
		dest = dest.add(8);
	}

	scalar::decode(string, dest).map_err(|error| error.shifted(input_len - string.len()))
}

#[test]
fn units() {
	let string = b"0123456789abcdef0123DE6789ABCDEF0415263748596a7B8C9dAeBfC0d1E2F304";
	let mut dest = [0u8; 33];
	unsafe { decode(string, dest.as_mut_ptr()).unwrap(); }
	assert_eq!(&dest, b"\x01\x23\x45\x67\x89\xab\xcd\xef\x01\x23\xde\x67\x89\xab\xcd\xef\x04\x15\x26\x37\x48\x59\x6a\x7b\x8c\x9d\xae\xbf\xc0\xd1\xe2\xf3\x04");
}
