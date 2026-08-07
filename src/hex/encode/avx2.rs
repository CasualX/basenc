#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn encode_16(data: __m128i, dest: *mut u8, charset: __m128i) {
	let mask = _mm_set1_epi8(0x0f);
	let lo = _mm_and_si128(data, mask);
	let hi = _mm_and_si128(_mm_srli_epi16(data, 4), mask);

	let digits1 = _mm_unpacklo_epi8(hi, lo);
	let digits2 = _mm_unpackhi_epi8(hi, lo);
	let ascii1 = _mm_shuffle_epi8(charset, digits1);
	let ascii2 = _mm_shuffle_epi8(charset, digits2);

	_mm_storeu_si128(dest as *mut __m128i, ascii1);
	_mm_storeu_si128(dest.add(16) as *mut __m128i, ascii2);
}

#[target_feature(enable = "avx2")]
pub unsafe fn encode(mut bytes: &[u8], mut dest: *mut u8, base: u8) -> *mut u8 {
	let charset = if base == b'A' {
		_mm_setr_epi8(
			b'0' as i8, b'1' as i8, b'2' as i8, b'3' as i8,
			b'4' as i8, b'5' as i8, b'6' as i8, b'7' as i8,
			b'8' as i8, b'9' as i8, b'A' as i8, b'B' as i8,
			b'C' as i8, b'D' as i8, b'E' as i8, b'F' as i8,
		)
	}
	else {
		_mm_setr_epi8(
			b'0' as i8, b'1' as i8, b'2' as i8, b'3' as i8,
			b'4' as i8, b'5' as i8, b'6' as i8, b'7' as i8,
			b'8' as i8, b'9' as i8, b'a' as i8, b'b' as i8,
			b'c' as i8, b'd' as i8, b'e' as i8, b'f' as i8,
		)
	};
	let charset256 = _mm256_broadcastsi128_si256(charset);
	let mask = _mm256_set1_epi8(0x0f);

	while bytes.len() >= 32 {
		let data = _mm256_loadu_si256(bytes.as_ptr() as *const __m256i);
		let lo = _mm256_and_si256(data, mask);
		let hi = _mm256_and_si256(_mm256_srli_epi16(data, 4), mask);

		// Unpack works within 128-bit lanes. Reassemble those lanes so each
		// output vector contains the encoding of 16 consecutive input bytes.
		let digits1 = _mm256_unpacklo_epi8(hi, lo);
		let digits2 = _mm256_unpackhi_epi8(hi, lo);
		let ascii1 = _mm256_shuffle_epi8(charset256, digits1);
		let ascii2 = _mm256_shuffle_epi8(charset256, digits2);
		let output1 = _mm256_permute2x128_si256::<0x20>(ascii1, ascii2);
		let output2 = _mm256_permute2x128_si256::<0x31>(ascii1, ascii2);

		_mm256_storeu_si256(dest as *mut __m256i, output1);
		_mm256_storeu_si256(dest.add(32) as *mut __m256i, output2);

		bytes = &bytes[32..];
		dest = dest.add(64);
	}

	if bytes.len() >= 16 {
		let data = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
		encode_16(data, dest, charset);
		bytes = &bytes[16..];
		dest = dest.add(32);
	}

	scalar::encode(bytes, dest, base)
}

#[test]
fn units() {
	let bytes = b"\x01\x23\x45\x67\x89\xAB\xCD\xEF\x01\x23\xde\x67\x89\xAB\xCD\xEF\x04\x15\x26\x37\x48\x59\x6a\x7b\x8c\x9d\xae\xbf\xc0\xd1\xe2\xf3\x04";
	let mut dest = [0u8; 66];
	unsafe { encode(bytes, dest.as_mut_ptr(), b'A'); }
	assert_eq!(&dest, b"0123456789ABCDEF0123DE6789ABCDEF0415263748596A7B8C9DAEBFC0D1E2F304");
}
