use basenc::*;

#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl Encoding, expected: &str) {
	assert_eq!(expected, encoding.encode_into(input, Padding::Strict, String::new()));
	assert_eq!(Ok(input), encoding.decode_into(expected.as_bytes(), Padding::Strict, Vec::new()).as_deref());
}

#[test]
fn rfc4648() {
	roundtrip(b"", &Base32Std, "");
	roundtrip(b"f", &Base32Std, "MY======");
	roundtrip(b"fo", &Base32Std, "MZXQ====");
	roundtrip(b"foo", &Base32Std, "MZXW6===");
	roundtrip(b"foob", &Base32Std, "MZXW6YQ=");
	roundtrip(b"fooba", &Base32Std, "MZXW6YTB");
	roundtrip(b"foobar", &Base32Std, "MZXW6YTBOI======");
	roundtrip(b"", &Base32Hex, "");
	roundtrip(b"f", &Base32Hex, "CO======");
	roundtrip(b"fo", &Base32Hex, "CPNG====");
	roundtrip(b"foo", &Base32Hex, "CPNMU===");
	roundtrip(b"foob", &Base32Hex, "CPNMUOG=");
	roundtrip(b"fooba", &Base32Hex, "CPNMUOJ1");
	roundtrip(b"foobar", &Base32Hex, "CPNMUOJ1E8======");
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
	smash(&Base32Std, &mut stack_buf);
	smash(&Base32Hex, &mut stack_buf);
	smash(&Base32Z, &mut stack_buf);
}
