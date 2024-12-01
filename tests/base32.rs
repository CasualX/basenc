
#[track_caller]
fn roundtrip(input: &[u8], encoding: &impl basenc::Encoding, expected: &str) {
	assert_eq!(expected, encoding.encode_into(input, basenc::Padding::Strict, String::new()));
	assert_eq!(Ok(input), encoding.decode_into(expected.as_bytes(), basenc::Padding::Strict, Vec::new()).as_deref());
}

#[test]
fn rfc4648() {
	roundtrip(b"", &basenc::Base32Std, "");
	roundtrip(b"f", &basenc::Base32Std, "MY======");
	roundtrip(b"fo", &basenc::Base32Std, "MZXQ====");
	roundtrip(b"foo", &basenc::Base32Std, "MZXW6===");
	roundtrip(b"foob", &basenc::Base32Std, "MZXW6YQ=");
	roundtrip(b"fooba", &basenc::Base32Std, "MZXW6YTB");
	roundtrip(b"foobar", &basenc::Base32Std, "MZXW6YTBOI======");
	roundtrip(b"", &basenc::Base32Hex, "");
	roundtrip(b"f", &basenc::Base32Hex, "CO======");
	roundtrip(b"fo", &basenc::Base32Hex, "CPNG====");
	roundtrip(b"foo", &basenc::Base32Hex, "CPNMU===");
	roundtrip(b"foob", &basenc::Base32Hex, "CPNMUOG=");
	roundtrip(b"fooba", &basenc::Base32Hex, "CPNMUOJ1");
	roundtrip(b"foobar", &basenc::Base32Hex, "CPNMUOJ1E8======");
}
