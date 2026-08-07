use basenc::*;

#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl Encoding, expected: &str) {
	assert_eq!(expected, encoding.encode_bytes_into(input, String::new()));
	assert_eq!(Ok(input), encoding.decode_bytes_into(expected.as_bytes(), Vec::new()).as_deref());
}

#[test]
fn rfc4648() {
	let base32std = Base32Std.pad(Padding::Required);
	roundtrip(b"", &base32std, "");
	roundtrip(b"f", &base32std, "MY======");
	roundtrip(b"fo", &base32std, "MZXQ====");
	roundtrip(b"foo", &base32std, "MZXW6===");
	roundtrip(b"foob", &base32std, "MZXW6YQ=");
	roundtrip(b"fooba", &base32std, "MZXW6YTB");
	roundtrip(b"foobar", &base32std, "MZXW6YTBOI======");
	let base32hex = Base32Hex.pad(Padding::Required);
	roundtrip(b"", &base32hex, "");
	roundtrip(b"f", &base32hex, "CO======");
	roundtrip(b"fo", &base32hex, "CPNG====");
	roundtrip(b"foo", &base32hex, "CPNMU===");
	roundtrip(b"foob", &base32hex, "CPNMUOG=");
	roundtrip(b"fooba", &base32hex, "CPNMUOJ1");
	roundtrip(b"foobar", &base32hex, "CPNMUOJ1E8======");
}

#[test]
fn padding_policies() {
	let forbidden = Base32Std.pad(Padding::Forbidden);
	assert_eq!(forbidden.encode(b"f"), "MY");
	assert_eq!(forbidden.decode("MY"), Ok(b"f".to_vec()));
	assert_eq!(forbidden.decode("MY======"), Err(Error::new(ErrorKind::InvalidCharacter, 2)));

	let optional = Base32Std.pad(Padding::Optional);
	assert_eq!(optional.encode(b"f"), "MY");
	assert_eq!(optional.decode("MY"), Ok(b"f".to_vec()));
	assert_eq!(optional.decode("MY======"), Ok(b"f".to_vec()));
	assert_eq!(optional.decode("MY======MZXQ===="), Err(Error::new(ErrorKind::InvalidCharacter, 8)));

	let standard = Base32Std.pad(Padding::Standard);
	assert_eq!(standard.encode(b"f"), "MY======");
	assert_eq!(standard.decode("MY"), Ok(b"f".to_vec()));
	assert_eq!(standard.decode("MY======"), Ok(b"f".to_vec()));
	assert_eq!(standard.decode("MY======MZXQ===="), Err(Error::new(ErrorKind::InvalidCharacter, 8)));
	assert_eq!(Base32Std.encode(b"f"), "MY======");
	assert_eq!(Base32Std.decode("MY"), Ok(b"f".to_vec()));
	assert_eq!(Base32Hex.encode(b"f"), "CO======");
	assert_eq!(Base32Z.encode(b"f"), "ca");
	assert_eq!(Base32Z.decode("ca"), Ok(b"f".to_vec()));
	assert_eq!(Base32Z.decode("ca======"), Err(Error::new(ErrorKind::InvalidCharacter, 2)));
	assert_eq!(Base32Z.pad(Padding::Standard).encode(b"f"), "ca======");

	let required = Base32Std.pad(Padding::Required);
	assert_eq!(required.encode(b"f"), "MY======");
	assert_eq!(required.decode("MY"), Err(Error::new(ErrorKind::IncorrectLength, 2)));
	assert_eq!(required.decode("MY======"), Ok(b"f".to_vec()));
	assert_eq!(required.decode("MY======MZXQ===="), Err(Error::new(ErrorKind::InvalidCharacter, 8)));

	let internal = Base32Std.pad(Padding::Internal);
	assert_eq!(internal.encode(b"f"), "MY");
	assert_eq!(internal.decode("MY"), Ok(b"f".to_vec()));
	assert_eq!(internal.decode("MY======"), Ok(b"f".to_vec()));
	assert_eq!(internal.decode("MY======MZXQ===="), Ok(b"ffo".to_vec()));
}

fn smash(encoding: &impl Encoding, input_buf: &mut [u8]) {
	let mut rng = urandom::new();

	for _ in 0..1000 {
		let len = rng.uniform(0..input_buf.len());
		rng.fill_bytes(&mut input_buf[..len]);

		let input = &input_buf[..len];
		let encoded = encoding.encode_bytes_into(input, String::new());
		let decoded = encoding.decode_bytes_into(encoded.as_bytes(), Vec::new()).unwrap();
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

#[test]
fn fully_dynamic_alphabet() {
	// Exercise every ASCII high-nibble row used by the SIMD reverse lookup.
	let charset = [
		0, 1, 2, 3, 16, 17, 18, 19, 32, 33, 34, 35, 48, 49, 50, 51,
		64, 65, 66, 67, 80, 81, 82, 83, 96, 97, 98, 99, 112, 113, 114, 115,
	];
	let dynamic = Base32::new(&charset);
	let standard = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

	for len in 0usize..=257 {
		let input: Vec<_> = (0..len).map(|i| i.wrapping_mul(73) as u8).collect();
		let expected: Vec<_> = Base32Std
			.encode(&input)
			.bytes()
			.map(|byte| {
				if byte == b'=' {
					byte
				}
				else {
					charset[standard.iter().position(|&candidate| candidate == byte).unwrap()]
				}
			})
			.collect();
		let encoded = dynamic.encode_into(&input, String::new());
		assert_eq!(encoded.as_bytes(), expected, "encoded length {len}");
		assert_eq!(
			Encoding::decode_bytes_into(&dynamic, encoded.as_bytes(), Vec::new()).as_deref(),
			Ok(input.as_slice()),
			"decoded length {len}"
		);

		let expected_padded: Vec<_> = Base32Std
			.pad(Padding::Required)
			.encode(&input)
			.bytes()
			.map(|byte| {
				if byte == b'=' {
					byte
				}
				else {
					charset[standard.iter().position(|&candidate| candidate == byte).unwrap()]
				}
			})
			.collect();
		let dynamic_padded = dynamic.pad(Padding::Required);
		let encoded_padded = dynamic_padded.encode_into(&input, String::new());
		assert_eq!(encoded_padded.as_bytes(), expected_padded, "padded encoded length {len}");
		assert_eq!(
			Encoding::decode_bytes_into(&dynamic_padded, encoded_padded.as_bytes(), Vec::new()).as_deref(),
			Ok(input.as_slice()),
			"padded decoded length {len}"
		);
	}
}

#[test]
fn simd_character_validation() {
	let input: Vec<_> = (0usize..40).map(|i| i.wrapping_mul(37) as u8).collect();
	let encoded = Base32Z.pad(NoPad).encode(&input).into_bytes();

	for index in 0..encoded.len() {
		let mut invalid = encoded.clone();
		invalid[index] = b'=';
		assert_eq!(
			Encoding::decode_bytes_into(&Base32Z.pad(NoPad), &invalid, Vec::new()),
			Err(Error::new(ErrorKind::InvalidCharacter, index)),
			"index {index}"
		);
	}

	let mut non_ascii = encoded;
	non_ascii[31] = 0x80;
	assert_eq!(Encoding::decode_bytes_into(&Base32Z, &non_ascii, Vec::new()), Err(Error::new(ErrorKind::InvalidCharacter, 31)));

	let mut noncanonical = [b'A'; 34];
	noncanonical[33] = b'B';
	assert_eq!(
		Encoding::decode_bytes_into(&Base32Std, &noncanonical, Vec::new()),
		Err(Error::new(ErrorKind::NonCanonical, 33)),
	);
}

#[test]
fn simd_stores_stay_within_estimated_output() {
	for len in 0usize..=65 {
		let input: Vec<_> = (0..len).map(|i| i.wrapping_mul(101) as u8).collect();
		let encoded_len = Base32::RATIO.estimate_encoded_len(len);
		let mut encoded = vec![0xa5; encoded_len + 1];
		let encoded_output = Base32Z.encode_into(&input, &mut encoded[..encoded_len]);
		let encoded_output = encoded_output.as_bytes().to_vec();
		assert_eq!(encoded[encoded_len], 0xa5, "encode length {len}");

		let decoded_len = Base32::RATIO.estimate_decoded_len(encoded_output.len());
		let mut decoded = vec![0xa5; decoded_len + 1];
		let decoded_output = Encoding::decode_bytes_into(&Base32Z, &encoded_output, &mut decoded[..decoded_len]).unwrap();
		assert_eq!(decoded_output, input, "decode length {len}");
		assert_eq!(decoded[decoded_len], 0xa5, "decode canary length {len}");
	}
}
