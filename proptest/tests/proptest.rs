use proptest::prelude::*;

fn parse_pad(i: i32) -> basenc::Padding {
	match i {
		0 => basenc::Padding::None,
		1 => basenc::Padding::Optional,
		2 => basenc::Padding::Strict,
		_ => unreachable!(),
	}
}

proptest! {
	#[test]
	fn base64_decode(s in "[a-zA-Z0-9+/]*") {
		let _ = basenc::Base64Std.decode(&s, basenc::NoPad);
	}

	#[test]
	fn base64_encode(s in "\\PC*", pad in 0..3) {
		let pad = parse_pad(pad);
		let encoded = basenc::Base64Std.encode(&s.as_bytes(), pad);
		let decoded = basenc::Base64Std.decode(&encoded, pad).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}

	#[test]
	fn base64url_decode(s in "[a-zA-Z0-9-_]*") {
		let _ = basenc::Base64Url.decode(&s, basenc::NoPad);
	}

	#[test]
	fn base64url_encode(s in "\\PC*", pad in 0..3) {
		let pad = parse_pad(pad);
		let encoded = basenc::Base64Url.encode(&s.as_bytes(), pad);
		let decoded = basenc::Base64Url.decode(&encoded, pad).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}

	#[test]
	fn base32_decode(s in "[A-Z2-7]*") {
		let _ = basenc::Base32Std.decode(&s, basenc::NoPad);
	}

	#[test]
	fn base32_encode(s in "\\PC*", pad in 0..3) {
		let pad = parse_pad(pad);
		let encoded = basenc::Base32Std.encode(&s.as_bytes(), pad);
		let decoded = basenc::Base32Std.decode(&encoded, pad).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}

	#[test]
	fn base32hex_decode(s in "[A-V0-9]*") {
		let _ = basenc::Base32Hex.decode(&s, basenc::NoPad);
	}

	#[test]
	fn base32hex_encode(s in "\\PC*", pad in 0..3) {
		let pad = parse_pad(pad);
		let encoded = basenc::Base32Hex.encode(&s.as_bytes(), pad);
		let decoded = basenc::Base32Hex.decode(&encoded, pad).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}

	#[test]
	fn hex_decode(s in "[0-9a-fA-F]*") {
		let _ = basenc::LowerHex.decode(&s);
	}

	#[test]
	fn hex_encode(s in "\\PC*") {
		let encoded = basenc::LowerHex.encode(&s.as_bytes());
		let decoded = basenc::LowerHex.decode(&encoded).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}
}
