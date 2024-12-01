#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

unsafe fn blend(a: __m128i, b: __m128i, mask: __m128i) -> __m128i {
	_mm_or_si128(_mm_andnot_si128(mask, a), _mm_and_si128(b, mask))
}

pub unsafe fn encode(src: *const u8, mut len: usize, dest: *mut u8, base: u8) {
	let _0x0f = _mm_set1_epi8(0xF);
	let _0x09 = _mm_set1_epi8(9);

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

	let mut src = src as *const __m128i;
	let mut dest = dest as *mut __m128i;
	while len >= 16 {
		let data = _mm_loadu_si128(src);

		// Split into digits
		let lo = _mm_and_si128(data, _0x0f);
		let hi = _mm_and_si128(_mm_srli_epi16(data, 4), _0x0f);
		let v1 = _mm_unpacklo_epi8(hi, lo);
		let v2 = _mm_unpackhi_epi8(hi, lo);

		// Convert to ASCII
		let (a1, a2);
		cfg_if::cfg_if! {
			if #[cfg(target_feature = "ssse3")] {
				a1 = _mm_shuffle_epi8(charset, v1);
				a2 = _mm_shuffle_epi8(charset, v2);
			}
			else {
				let base1 = blend(
					_mm_set1_epi8(b'0' as i8),
					_mm_set1_epi8(base as i8 - 10),
					_mm_cmpgt_epi8(v1, _0x09),
				);
				let base2 = blend(
					_mm_set1_epi8(b'0' as i8),
					_mm_set1_epi8(base as i8 - 10),
					_mm_cmpgt_epi8(v2, _0x09),
				);
				a1 = _mm_add_epi8(v1, base1);
				a2 = _mm_add_epi8(v2, base2);
			}
		}

		// Store result
		_mm_storeu_si128(dest, a1);
		_mm_storeu_si128(dest.offset(1), a2);

		src = src.offset(1);
		dest = dest.offset(2);
		len -= 16;
	}

	super::generic::encode(src as *const u8, len, dest as *mut u8, base);
}

#[test]
fn units() {
	let src = b"\x01\x23\x45\x67\x89\xAB\xCD\xEF\x01\x23\xde\x67\x89\xAB\xCD\xEF\x04";
	let mut dest = [0u8; 34];
	unsafe { encode(src.as_ptr(), src.len(), dest.as_mut_ptr(), b'A'); }
	assert_eq!(&dest, b"0123456789ABCDEF0123DE6789ABCDEF04");
	// panic!("{:x?}", core::str::from_utf8(&dest));
}
