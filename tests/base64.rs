use basenc::*;

#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl Encoding, expected: &str) {
	assert_eq!(expected.trim_end_matches("="), encoding.encode_into(input, String::new()));
	assert_eq!(Ok(input), encoding.decode_into(expected.as_bytes(), Vec::new()).as_deref());
}

#[track_caller]
fn error(string: &str, enc: &impl crate::Encoding, kind: ErrorKind, offset: usize) {
	let mut buf = [0u8; 64];
	assert_eq!(enc.decode_into(string.as_bytes(), &mut buf), Err(Error::new(kind, offset)));
}

#[test]
fn rfc4648() {
	// rfc4648 test vectors
	roundtrip(b"", &Base64Std, "");
	roundtrip(b"f", &Base64Std, "Zg==");
	roundtrip(b"fo", &Base64Std, "Zm8=");
	roundtrip(b"foo", &Base64Std, "Zm9v");
	roundtrip(b"foob", &Base64Std, "Zm9vYg==");
	roundtrip(b"fooba", &Base64Std, "Zm9vYmE=");
	roundtrip(b"foobar", &Base64Std, "Zm9vYmFy");
}

#[test]
fn wikipedia() {
	// Padding test vectors from wikipedia: https://en.wikipedia.org/wiki/Base64
	roundtrip(b"any carnal pleasure.", &Base64Std, "YW55IGNhcm5hbCBwbGVhc3VyZS4=");
	roundtrip(b"any carnal pleasure", &Base64Std, "YW55IGNhcm5hbCBwbGVhc3VyZQ==");
	roundtrip(b"any carnal pleasur", &Base64Std, "YW55IGNhcm5hbCBwbGVhc3Vy");
	roundtrip(b"any carnal pleasu", &Base64Std, "YW55IGNhcm5hbCBwbGVhc3U=");
	roundtrip(b"any carnal pleas", &Base64Std, "YW55IGNhcm5hbCBwbGVhcw==");
	roundtrip(b"pleasure.", &Base64Std, "cGxlYXN1cmUu", );
	roundtrip(b"leasure.", &Base64Std, "bGVhc3VyZS4=", );
	roundtrip(b"easure.", &Base64Std, "ZWFzdXJlLg==", );
	roundtrip(b"asure.", &Base64Std, "YXN1cmUu", );
	roundtrip(b"sure.", &Base64Std, "c3VyZS4=", );
}
#[test]
fn cwgem_test_base64_rb() {
	// Some test vectors I found with google: https://gist.github.com/cwgem/1209735
	// Note: Those tests use '+' in the url safe alphabet!

	roundtrip(b"Send reinforcements", &Base64Std, "U2VuZCByZWluZm9yY2VtZW50cw==");
	roundtrip(b"Now is the time for all good coders\nto learn Ruby", &Base64Std,
		"Tm93IGlzIHRoZSB0aW1lIGZvciBhbGwgZ29vZCBjb2RlcnMKdG8gbGVhcm4gUnVieQ==");
	roundtrip(b"This is line one\nThis is line two\nThis is line three\nAnd so on...\n", &Base64Std,
		"VGhpcyBpcyBsaW5lIG9uZQpUaGlzIGlzIGxpbmUgdHdvClRoaXMgaXMgbGluZSB0aHJlZQpBbmQgc28gb24uLi4K");
	roundtrip("テスト".as_bytes(), &Base64Std, "44OG44K544OI");

	roundtrip(b"", &Base64Std, "");
	roundtrip(b"\0", &Base64Std, "AA==");
	roundtrip(b"\0\0", &Base64Std, "AAA=");
	roundtrip(b"\0\0\0", &Base64Std, "AAAA");
	roundtrip(b"\xFF", &Base64Std, "/w==");
	roundtrip(b"\xFF\xFF", &Base64Std, "//8=");
	roundtrip(b"\xFF\xFF\xFF", &Base64Std, "////");
	roundtrip(b"\xff\xef", &Base64Std, "/+8=");

	let base64std_strict = Base64Std.pad(Padding::Required);
	error("^", &base64std_strict, ErrorKind::IncorrectLength, 1);
	error("A", &base64std_strict, ErrorKind::IncorrectLength, 1);
	error("A^", &base64std_strict, ErrorKind::IncorrectLength, 2);
	error("AA", &base64std_strict, ErrorKind::IncorrectLength, 2);
	error("AA=", &base64std_strict, ErrorKind::IncorrectLength, 3);
	error("AA===", &base64std_strict, ErrorKind::IncorrectLength, 5);
	error("AA=x", &base64std_strict, ErrorKind::InvalidCharacter, 2);
	error("AAA", &base64std_strict, ErrorKind::IncorrectLength, 3);
	error("AAA^", &base64std_strict, ErrorKind::InvalidCharacter, 3);
	error("AB==", &base64std_strict, ErrorKind::NonCanonical, 1);
	error("AAB=", &base64std_strict, ErrorKind::NonCanonical, 2);

	roundtrip(b"", &Base64Url, "");
	roundtrip(b"\0", &Base64Url, "AA");
	roundtrip(b"\0\0", &Base64Url, "AAA");
	roundtrip(b"\0\0\0", &Base64Url, "AAAA");
	roundtrip(b"\xFF", &Base64Url, "_w");
	roundtrip(b"\xFF\xFF", &Base64Url, "__8");
	roundtrip(b"\xFF\xFF\xFF", &Base64Url, "____");
	roundtrip(b"\xff\xef", &Base64Url, "_-8");
}

#[test]
fn padding_policies() {
	let forbidden = Base64Std.pad(Padding::Forbidden);
	assert_eq!(forbidden.encode(b"f"), "Zg");
	assert_eq!(forbidden.decode("Zg"), Ok(b"f".to_vec()));
	assert_eq!(forbidden.decode("Zg=="), Err(Error::new(ErrorKind::InvalidCharacter, 2)));

	let optional = Base64Std.pad(Padding::Optional);
	assert_eq!(optional.encode(b"f"), "Zg");
	assert_eq!(optional.decode("Zg"), Ok(b"f".to_vec()));
	assert_eq!(optional.decode_bytes(b"Zg"), Ok(b"f".to_vec()));
	assert_eq!(optional.decode_bytes_into(b"Zg", Vec::new()), Ok(b"f".to_vec()));
	assert_eq!(optional.decode("Zg=="), Ok(b"f".to_vec()));
	assert_eq!(optional.decode("Zg==Zm8="), Err(Error::new(ErrorKind::InvalidCharacter, 4)));

	let required = Base64Std.pad(Padding::Required);
	assert_eq!(required.encode(b"f"), "Zg==");
	assert_eq!(required.decode("Zg"), Err(Error::new(ErrorKind::IncorrectLength, 2)));
	assert_eq!(required.decode("Zg=="), Ok(b"f".to_vec()));
	assert_eq!(required.decode("Zg==Zm8="), Err(Error::new(ErrorKind::InvalidCharacter, 4)));

	let internal = Base64Std.pad(Padding::Internal);
	assert_eq!(internal.encode(b"f"), "Zg");
	assert_eq!(internal.decode("Zg"), Ok(b"f".to_vec()));
	assert_eq!(internal.decode("Zg=="), Ok(b"f".to_vec()));
	assert_eq!(internal.decode("Zg==Zm8="), Ok(b"ffo".to_vec()));

	// The first padded segment ends exactly at a SIMD block boundary.
	let segmented = "Zm9vYmFyYmF6Zg==Zm8=";
	assert_eq!(optional.decode(segmented), Err(Error::new(ErrorKind::InvalidCharacter, 16)));
	assert_eq!(internal.decode(segmented), Ok(b"foobarbazffo".to_vec()));

	// SIMD decoding can resume immediately after an internally padded segment.
	let segmented = "Zg==YWJjZGVmZ2hpamts";
	assert_eq!(internal.decode(segmented), Ok(b"fabcdefghijkl".to_vec()));
}

#[test]
fn proptest() {
	roundtrip("a￼\u{1cd00}ਏΣ".as_bytes(), &Base64Url, "Ye-_vPCctIDgqI_Oow");
}

#[test]
fn custom_alphabets() {
	roundtrip(&[0xfb, 0xef, 0xbe, 0xfb, 0xef, 0xbe, 0xfb, 0xef, 0xbe, 0xfb, 0xef, 0xbe], &Base64::new(b'>', b'/'), ">>>>>>>>>>>>>>>>");
	roundtrip(&[0xff; 12], &Base64::new(b'+', b'?'), "????????????????");
}

fn smash(encoding: &impl Encoding, input_buf: &mut [u8]) {
	let mut rng = urandom::new();

	for _ in 0..1000 {
		let len = rng.uniform(0..input_buf.len());
		rng.fill_bytes(&mut input_buf[..len]);

		let input = &input_buf[..len];
		let encoded = encoding.encode_into(input, String::new());
		let decoded = encoding.decode_into(encoded.as_bytes(), Vec::new()).unwrap();
		assert_eq!(input, decoded);
	}
}

#[test]
fn random() {
	let mut stack_buf = [0u8; 1024];
	smash(&Base64Std.pad(NoPad), &mut stack_buf);
	smash(&Base64Url.pad(NoPad), &mut stack_buf);
}

#[test]
fn simd_character_validation() {
	fn check(encoding: &impl Encoding, char62: u8, char63: u8) {
		for byte in 0..=u8::MAX {
			let mut string = [b'A'; 32];
			string[8] = byte;
			let valid = byte.is_ascii_uppercase()
				|| byte.is_ascii_lowercase()
				|| byte.is_ascii_digit()
				|| byte == char62
				|| byte == char63;
			let result = encoding.decode_into(&string, Vec::new()).map(|_| ());
			let expected = if valid {
				Ok(())
			}
			else {
				Err(Error::new(ErrorKind::InvalidCharacter, 8))
			};
			assert_eq!(result, expected, "byte {byte:#04x}");
		}
	}

	check(&Base64Std.pad(NoPad), b'+', b'/');
	check(&Base64Url.pad(NoPad), b'-', b'_');

	let mut noncanonical = [b'A'; 34];
	noncanonical[33] = b'B';
	assert_eq!(
		Encoding::decode_into(&Base64Std, &noncanonical, Vec::new()),
		Err(Error::new(ErrorKind::NonCanonical, 33)),
	);
	assert_eq!(
		Encoding::decode_into(&Base64Url, &noncanonical, Vec::new()),
		Err(Error::new(ErrorKind::NonCanonical, 33)),
	);
}

#[test]
fn simd_stores_stay_within_estimated_output() {
	const CANARY: u8 = 0xa5;
	let encoding = Base64Std.pad(NoPad);

	for len in 0usize..=256 {
		let input: Vec<_> = (0..len).map(|i| (i as u8).wrapping_mul(37)).collect();
		let encoded_capacity = (len.saturating_add(2) / 3) * 4;
		let mut encoded_storage = [CANARY; 400];
		let encoded = encoding
			.encode_into(&input, &mut encoded_storage[..encoded_capacity])
			.to_owned();
		assert!(encoded_storage[encoded_capacity..].iter().all(|&byte| byte == CANARY));

		let decoded_capacity = (encoded.len().saturating_add(3) / 4) * 3;
		let mut decoded_storage = [CANARY; 300];
		let decoded = encoding
			.decode_into(&encoded, &mut decoded_storage[..decoded_capacity])
			.unwrap();
		assert_eq!(decoded, input);
		assert!(decoded_storage[decoded_capacity..].iter().all(|&byte| byte == CANARY));
	}
}

#[test]
fn dedicated_standard_matches_generic_alphabet() {
	assert_eq!(core::mem::size_of::<Base64Std>(), 0);
	let generic = Base64::new(b'+', b'/');

	for pad in [Padding::Forbidden, Padding::Optional, Padding::Required, Padding::Internal] {
		let dedicated = Base64Std.pad(pad);
		let generic = generic.pad(pad);
		for len in 0usize..=256 {
			let input: Vec<_> = (0..len).map(|i| (i as u8).wrapping_mul(73)).collect();
			let encoded = dedicated.encode(&input);
			assert_eq!(generic.encode(&input), encoded);
			assert_eq!(dedicated.decode(&encoded), generic.decode(&encoded));
		}
	}
}
