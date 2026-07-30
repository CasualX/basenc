#![feature(test)]

extern crate test;

use basenc::Encoding as _;
use test::{black_box, Bencher};

const INPUT: &[u8] = include_bytes!("../../../src/base64.rs");

#[bench]
fn basenc_encode(b: &mut Bencher) {
	let input = black_box(INPUT);
	let encoding = basenc::Base64Std.pad(basenc::Padding::Required);
	let output_len = basenc::Base64Std::RATIO.estimate_encoded_len(input.len());
	let mut output = vec![0; output_len];

	b.bytes = input.len() as u64;
	b.iter(|| {
		let len = encoding.encode_into(input, output.as_mut_slice()).len();
		black_box(len);
		black_box(&output);
	});
}

#[bench]
fn base64_turbo_encode(b: &mut Bencher) {
	let input = black_box(INPUT);
	let engine = base64_turbo::STANDARD;
	let mut output = vec![0; engine.encoded_len(input.len())];

	b.bytes = input.len() as u64;
	b.iter(|| {
		black_box(engine.encode_into(input, &mut output).unwrap());
		black_box(&output);
	});
}

#[bench]
fn basenc_decode(b: &mut Bencher) {
	let encoding = basenc::Base64Std.pad(basenc::Padding::Required);
	let encoded = black_box(encoding.encode(INPUT));
	let output_len = basenc::Base64Std::RATIO.estimate_decoded_len(encoded.len());
	let mut output = vec![0; output_len];

	b.bytes = encoded.len() as u64;
	b.iter(|| {
		let len = encoding.decode_into(&encoded, output.as_mut_slice()).unwrap().len();
		black_box(len);
		black_box(&output);
	});
}

#[bench]
fn base64_turbo_decode(b: &mut Bencher) {
	let engine = base64_turbo::STANDARD;
	let encoded = black_box(engine.encode(INPUT));
	let mut output = vec![0; engine.estimate_decoded_len(encoded.len())];

	b.bytes = encoded.len() as u64;
	b.iter(|| {
		black_box(engine.decode_into(&encoded, &mut output).unwrap());
		black_box(&output);
	});
}
