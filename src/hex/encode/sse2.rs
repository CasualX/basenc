#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

#[inline]
#[target_feature(enable = "sse2")]
unsafe fn hex2ascii(v: __m128i, base: u8) -> __m128i {
	let mask = _mm_cmpgt_epi8(v, _mm_set1_epi8(9));

	let ascii_base = _mm_or_si128(
		_mm_andnot_si128(mask, _mm_set1_epi8(b'0' as i8)),
		_mm_and_si128(mask, _mm_set1_epi8(base as i8 - 10)),
	);
	return _mm_add_epi8(ascii_base, v);
}

#[target_feature(enable = "sse2")]
pub unsafe fn encode(mut bytes: &[u8], dest: *mut u8, base: u8) -> *mut u8 {
	let _0x0f = _mm_set1_epi8(0xF);

	let mut dest = dest as *mut __m128i;
	while bytes.len() >= 16 {
		let data = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);

		// Split into digits
		let lo = _mm_and_si128(data, _0x0f);
		let hi = _mm_and_si128(_mm_srli_epi16(data, 4), _0x0f);
		let v1 = _mm_unpacklo_epi8(hi, lo);
		let v2 = _mm_unpackhi_epi8(hi, lo);

		// Convert to ASCII
		let a1 = hex2ascii(v1, base);
		let a2 = hex2ascii(v2, base);

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
