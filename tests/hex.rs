use basenc::*;

#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl Encoding, expected: &str) {
	assert_eq!(expected, encoding.encode_into(input, NoPad, String::new()));
	assert_eq!(Ok(input), encoding.decode_into(expected.as_bytes(), NoPad, Vec::new()).as_deref());
}

#[test]
fn rfc4648() {
	// rfc4648 test vectors
	roundtrip(b"", &UpperHex, "");
	roundtrip(b"f", &UpperHex, "66");
	roundtrip(b"fo", &UpperHex, "666F");
	roundtrip(b"foo", &UpperHex, "666F6F");
	roundtrip(b"foob", &UpperHex, "666F6F62");
	roundtrip(b"fooba", &UpperHex, "666F6F6261");
	roundtrip(b"foobar", &UpperHex, "666F6F626172");
}

#[test]
fn stuff() {
	let bytes = &[0x5a, 0xcf, 0xfd, 0xa7, 0xca, 0x3e, 0x37, 0x3d, 0x4a, 0x11][..];
	roundtrip(bytes, &LowerHex, "5acffda7ca3e373d4a11");
	roundtrip(bytes, &UpperHex, "5ACFFDA7CA3E373D4A11");
	assert_eq!(LowerHex.decode_into("5ACfFda7cA3e373D4a11", &mut [0u8; 16]), Ok(bytes));
	assert_eq!(UpperHex.decode_into("5acFfDA7Ca3E373d4A11", &mut [0u8; 16]), Ok(bytes));
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
	smash(&LowerHex, &mut stack_buf);
	smash(&UpperHex, &mut stack_buf);
}
