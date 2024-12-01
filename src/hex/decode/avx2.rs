// https://github.com/zbjornson/fast-hex

use std::arch::x86_64::*;

unsafe fn unhex(value: __m256i) -> __m256i {
	let and15 = _mm256_and_si256(value, _mm256_set1_epi16(0x0F));

	let sr6 = _mm256_srai_epi16(value, 6);
	let mul = _mm256_maddubs_epi16(sr6, _mm256_set1_epi16(9));

	let add = _mm256_add_epi16(mul, and15);
	return add;
}

unsafe fn nib2bytes(a1: __m256i, b1: __m256i, a2: __m256i, b2: __m256i) -> __m256i {
	let a4_1 = _mm256_slli_epi16(a1, 4);
	let a4_2 = _mm256_slli_epi16(a2, 4);

	let a4orb_1 = _mm256_or_si256(a4_1, b1);
	let a4orb_2 = _mm256_or_si256(a4_2, b2);

	let pck1 = _mm256_packus_epi16(a4orb_1, a4orb_2); // lo1 lo2 hi1 hi2

	let pck64 = _mm256_permute4x64_epi64::<0b11_01_10_00>(pck1);

	return pck64;
}

pub unsafe fn decode(src: *const u8, mut len: usize, dest: *mut u8) {
	let a_mask = _mm256_setr_epi8(
		0, -1, 2, -1, 4, -1, 6, -1, 8, -1, 10, -1, 12, -1, 14, -1,
		0, -1, 2, -1, 4, -1, 6, -1, 8, -1, 10, -1, 12, -1, 14, -1);
	let b_mask = _mm256_setr_epi8(
		1, -1, 3, -1, 5, -1, 7, -1, 9, -1, 11, -1, 13, -1, 15, -1,
		1, -1, 3, -1, 5, -1, 7, -1, 9, -1, 11, -1, 13, -1, 15, -1);

	let mut src = src as *const __m256i;
	let mut dest = dest as *mut __m256i;
	while len >= 32 {
		let av1 = _mm256_lddqu_si256(src.offset(0));
		let av2 = _mm256_lddqu_si256(src.offset(1));

		eprintln!("av1={:x?}", av1);
		eprintln!("av2={:x?}", av2);

		let a1 = _mm256_shuffle_epi8(av1, a_mask);
		let b1 = _mm256_shuffle_epi8(av1, b_mask);
		let a2 = _mm256_shuffle_epi8(av2, a_mask);
		let b2 = _mm256_shuffle_epi8(av2, b_mask);

		eprintln!("a1={:x?}", a1);
		eprintln!("b1={:x?}", b1);
		eprintln!("a2={:x?}", a2);
		eprintln!("b2={:x?}", b2);

		let a1 = unhex(a1);
		let b1 = unhex(b1);
		let a2 = unhex(a2);
		let b2 = unhex(b2);

		let bytes = nib2bytes(a1, b1, a2, b2);

		_mm256_storeu_si256(dest, bytes);

		src = src.offset(2);
		dest = dest.offset(1);
		len -= 32;
	}

	let _ = super::decode::decode(src as *const u8, len, dest as *mut u8);
}

// #[test]
// fn test() {
// 	let input = b"0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF";
// 	let mut output = [0u8; 32];

// 	unsafe {
// 		decode(input.as_ptr(), output.len(), output.as_mut_ptr());
// 	}

// 	println!("{:x?}", &output);
// 	panic!();
// }
