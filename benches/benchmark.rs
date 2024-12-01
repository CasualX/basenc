#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

#[bench]
fn basenc(b: &mut Bencher) {
	let input = black_box(include_str!("../src/base64.rs").as_bytes());
	b.iter(|| {
		let encoded = basenc::Base64StdNoPad.encode(input);
		let decoded = basenc::Base64StdNoPad.decode(&encoded).unwrap();
		assert_eq!(input, decoded);
	});
}

#[bench]
fn simple_base64(b: &mut Bencher) {
	let input = black_box(include_str!("../src/base64.rs").as_bytes());
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
