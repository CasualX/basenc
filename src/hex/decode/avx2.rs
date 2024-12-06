// https://github.com/zbjornson/fast-hex
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "avx2")]
unsafe fn unhex(value: __m256i) -> __m256i {
	let and15 = _mm256_and_si256(value, _mm256_set1_epi16(0x0F));

	let sr6 = _mm256_srai_epi16(value, 6);
	let mul = _mm256_maddubs_epi16(sr6, _mm256_set1_epi16(9));

	let add = _mm256_add_epi16(mul, and15);
	return add;
}

#[target_feature(enable = "avx2")]
unsafe fn nib2bytes(a1: __m256i, b1: __m256i, a2: __m256i, b2: __m256i) -> __m256i {
	let a4_1 = _mm256_slli_epi16(a1, 4);
	let a4_2 = _mm256_slli_epi16(a2, 4);

	let a4orb_1 = _mm256_or_si256(a4_1, b1);
	let a4orb_2 = _mm256_or_si256(a4_2, b2);

	let pck1 = _mm256_packus_epi16(a4orb_1, a4orb_2); // lo1 lo2 hi1 hi2

	let pck64 = _mm256_permute4x64_epi64::<0b11_01_10_00>(pck1);

	return pck64;
}

#[target_feature(enable = "avx2")]
pub unsafe fn decode(mut string: &[u8], mut dest: *mut u8) -> Result<*mut u8, crate::Error> {
	let a_mask = _mm256_setr_epi8(
		0, -1, 2, -1, 4, -1, 6, -1, 8, -1, 10, -1, 12, -1, 14, -1,
		0, -1, 2, -1, 4, -1, 6, -1, 8, -1, 10, -1, 12, -1, 14, -1);
	let b_mask = _mm256_setr_epi8(
		1, -1, 3, -1, 5, -1, 7, -1, 9, -1, 11, -1, 13, -1, 15, -1,
		1, -1, 3, -1, 5, -1, 7, -1, 9, -1, 11, -1, 13, -1, 15, -1);

	while string.len() >= 64 {
		let src = string.as_ptr() as *const __m256i;
		let av1 = _mm256_lddqu_si256(src);
		let av2 = _mm256_lddqu_si256(src.offset(1));

		let a1 = _mm256_shuffle_epi8(av1, a_mask);
		let b1 = _mm256_shuffle_epi8(av1, b_mask);
		let a2 = _mm256_shuffle_epi8(av2, a_mask);
		let b2 = _mm256_shuffle_epi8(av2, b_mask);

		let a1 = unhex(a1);
		let b1 = unhex(b1);
		let a2 = unhex(a2);
		let b2 = unhex(b2);

		let bytes = nib2bytes(a1, b1, a2, b2);

		_mm256_storeu_si256(dest as *mut __m256i, bytes);

		string = &string[64..];
		dest = dest.offset(32);
	}

	scalar::decode(string, dest)
}
