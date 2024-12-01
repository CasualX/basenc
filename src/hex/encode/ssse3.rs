#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[target_feature(enable = "ssse3")]
pub unsafe fn encode(mut bytes: &[u8], dest: *mut u8, base: u8) -> *mut u8 {
	let _0x0f = _mm_set1_epi8(0xF);

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

	let mut dest = dest as *mut __m128i;
	while bytes.len() >= 16 {
		let data = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);

		// Split into digits
		let lo = _mm_and_si128(data, _0x0f);
		let hi = _mm_and_si128(_mm_srli_epi16(data, 4), _0x0f);
		let v1 = _mm_unpacklo_epi8(hi, lo);
		let v2 = _mm_unpackhi_epi8(hi, lo);

		// Convert to ASCII
		let a1 = _mm_shuffle_epi8(charset, v1);
		let a2 = _mm_shuffle_epi8(charset, v2);

		// Store result
		_mm_storeu_si128(dest, a1);
		_mm_storeu_si128(dest.offset(1), a2);

		dest = dest.offset(2);
		bytes = &bytes[16..];
	}

	scalar::encode(bytes, dest as *mut u8, base)
}

#[test]
fn units() {
	let bytes = b"\x01\x23\x45\x67\x89\xAB\xCD\xEF\x01\x23\xde\x67\x89\xAB\xCD\xEF\x04";
	let mut dest = [0u8; 34];
	unsafe { encode(bytes, dest.as_mut_ptr(), b'A'); }
	assert_eq!(&dest, b"0123456789ABCDEF0123DE6789ABCDEF04");
	// panic!("{:x?}", str::from_utf8(&dest));
}
