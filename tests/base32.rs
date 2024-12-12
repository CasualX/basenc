use basenc::*;

#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl Encoding, expected: &str) {
	assert_eq!(expected, encoding.encode_into(input, String::new()));
	assert_eq!(Ok(input), encoding.decode_into(expected.as_bytes(), Vec::new()).as_deref());
}

#[test]
fn rfc4648() {
	let base32std = Base32Std.pad(Padding::Strict);
	roundtrip(b"", &base32std, "");
	roundtrip(b"f", &base32std, "MY======");
	roundtrip(b"fo", &base32std, "MZXQ====");
	roundtrip(b"foo", &base32std, "MZXW6===");
	roundtrip(b"foob", &base32std, "MZXW6YQ=");
	roundtrip(b"fooba", &base32std, "MZXW6YTB");
	roundtrip(b"foobar", &base32std, "MZXW6YTBOI======");
	let base32hex = Base32Hex.pad(Padding::Strict);
	roundtrip(b"", &base32hex, "");
	roundtrip(b"f", &base32hex, "CO======");
	roundtrip(b"fo", &base32hex, "CPNG====");
	roundtrip(b"foo", &base32hex, "CPNMU===");
	roundtrip(b"foob", &base32hex, "CPNMUOG=");
	roundtrip(b"fooba", &base32hex, "CPNMUOJ1");
	roundtrip(b"foobar", &base32hex, "CPNMUOJ1E8======");
}

fn smash(encoding: &impl Encoding, input_buf: &mut [u8]) {
	let mut rng = urandom::new();

	for _ in 0..1000 {
		let len = rng.range(0..input_buf.len());
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
	smash(&Base32Std.pad(NoPad), &mut stack_buf);
	smash(&Base32Hex.pad(NoPad), &mut stack_buf);
	smash(&Base32Z.pad(NoPad), &mut stack_buf);
}
