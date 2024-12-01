#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

#[bench]
fn basenc_base64(b: &mut Bencher) {
	let input = black_box(include_str!("../src/base64.rs").as_bytes());
	b.bytes = input.len() as u64;
	b.iter(|| {
		let encoded = basenc::Base64Std.encode(input, basenc::NoPad);
		let decoded = basenc::Base64Std.decode(&encoded, basenc::NoPad).unwrap();
		assert_eq!(input, decoded);
	});
}

#[bench]
fn simple_base64(b: &mut Bencher) {
	let input = black_box(include_str!("../src/base64.rs").as_bytes());
	b.bytes = input.len() as u64;
	b.iter(|| {
		let encoded = simple_base64::encode_engine(input, &simple_base64::engine::general_purpose::STANDARD_NO_PAD);
		let decoded = simple_base64::decode_engine(encoded, &simple_base64::engine::general_purpose::STANDARD_NO_PAD).unwrap();
		assert_eq!(input, decoded);
	});
}

#[bench]
fn basenc_hex(b: &mut Bencher) {
	let input = black_box(include_str!("../src/hex.rs").as_bytes());
	b.bytes = input.len() as u64;
	b.iter(|| {
		let encoded = basenc::LowerHex.encode(input);
		let decoded = basenc::LowerHex.decode(&encoded).unwrap();
		assert_eq!(input, decoded);
	});
}

#[bench]
fn basenc_base32(b: &mut Bencher) {
	let input = black_box(include_str!("../src/base32.rs").as_bytes());
	b.bytes = input.len() as u64;
	b.iter(|| {
		let encoded = basenc::Base32Std.encode(input, basenc::NoPad);
		let decoded = basenc::Base32Std.decode(&encoded, basenc::NoPad).unwrap();
		assert_eq!(input, decoded);
	});
}
