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
		let _ = basenc::Base64Std.pad(basenc::NoPad).decode(&s);
	}

	#[test]
	fn base64_encode(s in "\\PC*", pad in 0..3) {
		let encoding = basenc::Base64Std.pad(parse_pad(pad));
		let encoded = encoding.encode(&s.as_bytes());
		let decoded = encoding.decode(&encoded).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}

	#[test]
	fn base64url_decode(s in "[a-zA-Z0-9-_]*") {
		let _ = basenc::Base64Url.pad(basenc::NoPad).decode(&s);
	}

	#[test]
	fn base64url_encode(s in "\\PC*", pad in 0..3) {
		let encoding = basenc::Base64Url.pad(parse_pad(pad));
		let encoded = encoding.encode(&s.as_bytes());
		let decoded = encoding.decode(&encoded).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}

	#[test]
	fn base32_decode(s in "[A-Z2-7]*") {
		let _ = basenc::Base32Std.pad(basenc::NoPad).decode(&s);
	}

	#[test]
	fn base32_encode(s in "\\PC*", pad in 0..3) {
		let encoding = basenc::Base32Std.pad(parse_pad(pad));
		let encoded = encoding.encode(&s.as_bytes());
		let decoded = encoding.decode(&encoded).unwrap();
		assert_eq!(s.as_bytes(), decoded);
	}

	#[test]
	fn base32hex_decode(s in "[A-V0-9]*") {
		let _ = basenc::Base32Hex.pad(basenc::NoPad).decode(&s);
	}

	#[test]
	fn base32hex_encode(s in "\\PC*", pad in 0..3) {
		let encoding = basenc::Base32Hex.pad(parse_pad(pad));
		let encoded = encoding.encode(&s.as_bytes());
		let decoded = encoding.decode(&encoded).unwrap();
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
