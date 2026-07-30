#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

#[bench]
fn base64_encode(b: &mut Bencher) {
	let input = black_box(include_bytes!("../src/base64.rs").as_slice());
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
fn base64_decode(b: &mut Bencher) {
	let encoding = basenc::Base64Std.pad(basenc::NoPad);
	let encoded = black_box(encoding.encode(include_bytes!("../src/base64.rs")));
	let mut output = encoding.decode(&encoded).unwrap();
	output.clear();

	b.bytes = encoded.len() as u64;
	b.iter(|| {
		output.clear();
		black_box(encoding.decode_into(&encoded, &mut output).unwrap());
	});
}
