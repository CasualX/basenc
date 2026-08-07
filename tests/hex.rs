use basenc::*;

#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl Encoding, expected: &str) {
	assert_eq!(expected, encoding.encode_into(input, String::new()));
	assert_eq!(Ok(input), encoding.decode_into(expected.as_bytes(), Vec::new()).as_deref());
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
	assert_eq!(LowerHex.decode_bytes(b"5ACfFda7cA3e373D4a11").as_deref(), Ok(bytes));
	assert_eq!(UpperHex.decode_bytes_into(b"5acFfDA7Ca3E373d4A11", &mut [0u8; 16]), Ok(bytes));
	assert_eq!(LowerHex.decode_bytes(b"00\xff0"), Err(Error::new(ErrorKind::InvalidCharacter, 2)));
	let error = LowerHex.decode("0").unwrap_err();
	assert_eq!(error.kind, ErrorKind::IncorrectLength);
	assert_eq!(error.offset, 1);
	assert_eq!(error.to_string(), "incorrect length at offset 1");
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
	smash(&LowerHex, &mut stack_buf);
	smash(&UpperHex, &mut stack_buf);
}

#[test]
fn simd_character_validation() {
	for len in [16, 32, 64, 96] {
		let mut string = vec![b'0'; len];
		let mut output = [0u8; 48];

		for index in 0..len {
			for byte in 0..=u8::MAX {
				let valid = byte.is_ascii_hexdigit();
				string[index] = byte;
				let result = LowerHex.decode_bytes_into(&string, &mut output).map(|_| ());
				let expected = if valid {
					Ok(())
				}
				else {
					Err(Error::new(ErrorKind::InvalidCharacter, index))
				};
				assert_eq!(result, expected, "length {len}, index {index}, byte {byte:#04x}");
			}
			string[index] = b'0';
		}
	}

	assert_eq!(
		LowerHex.decode_bytes(&[b'0'; 33]),
		Err(Error::new(ErrorKind::IncorrectLength, 33)),
	);
}

#[test]
fn simd_stores_stay_within_estimated_output() {
	const CANARY: u8 = 0xa5;

	for len in 0usize..=256 {
		let input: Vec<_> = (0..len).map(|i| (i as u8).wrapping_mul(37)).collect();
		let encoded_capacity = len * 2;
		let mut encoded_storage = [CANARY; 520];
		let encoded = LowerHex
			.encode_into(&input, &mut encoded_storage[..encoded_capacity])
			.to_owned();
		assert!(encoded_storage[encoded_capacity..].iter().all(|&byte| byte == CANARY));

		let decoded_capacity = encoded.len() / 2;
		let mut decoded_storage = [CANARY; 260];
		let decoded = LowerHex
			.decode_into(&encoded, &mut decoded_storage[..decoded_capacity])
			.unwrap();
		assert_eq!(decoded, input);
		assert!(decoded_storage[decoded_capacity..].iter().all(|&byte| byte == CANARY));
	}
}
