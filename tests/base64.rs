use basenc::*;

#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl Encoding, expected: &str) {
	assert_eq!(expected.trim_end_matches("="), encoding.encode_into(input, Padding::Optional, String::new()));
	assert_eq!(Ok(input), encoding.decode_into(expected.as_bytes(), Padding::Optional, Vec::new()).as_deref());
}

#[track_caller]
fn error(string: &str, enc: &impl crate::Encoding, err: Error) {
	let mut buf = [0u8; 64];
	assert_eq!(enc.decode_into(string.as_bytes(), Padding::Strict, &mut buf), Err(err));
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

	error("^", &Base64Std, Error::IncorrectLength);
	error("A", &Base64Std, Error::IncorrectLength);
	error("A^", &Base64Std, Error::IncorrectLength);
	error("AA", &Base64Std, Error::IncorrectLength);
	error("AA=", &Base64Std, Error::IncorrectLength);
	error("AA===", &Base64Std, Error::IncorrectLength);
	error("AA=x", &Base64Std, Error::InvalidCharacter);
	error("AAA", &Base64Std, Error::IncorrectLength);
	error("AAA^", &Base64Std, Error::InvalidCharacter);
	error("AB==", &Base64Std, Error::NonCanonical);
	error("AAB=", &Base64Std, Error::NonCanonical);

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
fn proptest() {
	roundtrip("a￼\u{1cd00}ਏΣ".as_bytes(), &Base64Url, "Ye-_vPCctIDgqI_Oow");
}

fn smash(encoding: &impl Encoding, input_buf: &mut [u8]) {
	let mut rng = urandom::new();

	for _ in 0..1000 {
		let len = rng.range(0..input_buf.len());
		rng.fill_bytes(&mut input_buf[..len]);

		let input = &input_buf[..len];
		let encoded = encoding.encode_into(input, NoPad, String::new());
		let decoded = encoding.decode_into(encoded.as_bytes(), NoPad, Vec::new()).unwrap();
		assert_eq!(input, decoded);
	}
}

#[test]
fn random() {
	let mut stack_buf = [0u8; 1024];
	smash(&Base64Std, &mut stack_buf);
	smash(&Base64Url, &mut stack_buf);
}
