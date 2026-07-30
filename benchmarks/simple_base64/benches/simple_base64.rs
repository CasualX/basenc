#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

const INPUT: &[u8] = include_bytes!("../../../src/base64.rs");

#[bench]
fn basenc_encode(b: &mut Bencher) {
	let input = black_box(INPUT);
	let encoding = basenc::Base64Std.pad(basenc::NoPad);
	let mut output = encoding.encode(input);
	output.clear();

	b.bytes = input.len() as u64;
	b.iter(|| {
		output.clear();
		black_box(encoding.encode_into(input, &mut output));
	});
}

#[bench]
fn simple_base64_encode(b: &mut Bencher) {
	let input = black_box(INPUT);
	let engine = &simple_base64::engine::general_purpose::STANDARD_NO_PAD;
	let mut output = simple_base64::encode_engine(input, engine);
	output.clear();

	b.bytes = input.len() as u64;
	b.iter(|| {
		output.clear();
		simple_base64::encode_engine_string(input, &mut output, engine);
		black_box(&output);
	});
}

#[bench]
fn basenc_decode(b: &mut Bencher) {
	let encoding = basenc::Base64Std.pad(basenc::NoPad);
	let encoded = black_box(encoding.encode(INPUT));
	let mut output = encoding.decode(&encoded).unwrap();
	output.clear();

	b.bytes = encoded.len() as u64;
	b.iter(|| {
		output.clear();
		black_box(encoding.decode_into(&encoded, &mut output).unwrap());
	});
}

#[bench]
fn simple_base64_decode(b: &mut Bencher) {
	let engine = &simple_base64::engine::general_purpose::STANDARD_NO_PAD;
	let encoded = black_box(simple_base64::encode_engine(INPUT, engine));
	let mut output = simple_base64::decode_engine(&encoded, engine).unwrap();
	output.clear();

	b.bytes = encoded.len() as u64;
	b.iter(|| {
		output.clear();
		simple_base64::decode_engine_vec(&encoded, &mut output, engine).unwrap();
		black_box(&output);
	});
}
