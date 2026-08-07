#![feature(test)]
extern crate test;

fn input(len: usize) -> Vec<u8> {
	(0..len).map(|index| index.wrapping_mul(37) as u8).collect()
}

const SCATTERED: basenc::Base32 = basenc::Base32::new(&[
	0, 1, 2, 3, 16, 17, 18, 19, 32, 33, 34, 35, 48, 49, 50, 51,
	64, 65, 66, 67, 80, 81, 82, 83, 96, 97, 98, 99, 112, 113, 114, 115,
]);

macro_rules! base32_bench {
	($encode:ident, $decode:ident, $len:expr) => {
		#[bench]
		fn $encode(b: &mut test::Bencher) {
			let input = test::black_box(input($len));
			// z-base-32 ensures the benchmark exercises a non-contiguous,
			// caller-provided alphabet rather than an RFC-specific fast path.
			let encoding = basenc::Base32Z.pad(basenc::NoPad);
			let mut output = encoding.encode(&input);
			output.clear();

			b.bytes = input.len() as u64;
			b.iter(|| {
				output.clear();
				test::black_box(encoding.encode_into(&input, &mut output));
			});
		}

		#[bench]
		fn $decode(b: &mut test::Bencher) {
			let input = input($len);
			let encoding = basenc::Base32Z.pad(basenc::NoPad);
			let encoded = test::black_box(encoding.encode(&input));
			let mut output = encoding.decode(&encoded).unwrap();
			output.clear();

			// Report throughput in decoded bytes so encode and decode are comparable.
			b.bytes = input.len() as u64;
			b.iter(|| {
				output.clear();
				test::black_box(encoding.decode_into(&encoded, &mut output).unwrap());
			});
		}
	};
}

base32_bench!(encode_10, decode_10, 10);
base32_bench!(encode_20, decode_20, 20);
base32_bench!(encode_4k, decode_4k, 4 * 1024);
base32_bench!(encode_1m, decode_1m, 1024 * 1024);

#[bench]
fn encode_scattered_4k(b: &mut test::Bencher) {
	let input = test::black_box(input(4 * 1024));
	let mut output = SCATTERED.encode(&input);
	output.clear();

	b.bytes = input.len() as u64;
	b.iter(|| {
		output.clear();
		test::black_box(SCATTERED.encode_into(&input, &mut output));
	});
}

#[bench]
fn decode_scattered_4k(b: &mut test::Bencher) {
	let input = input(4 * 1024);
	let encoded = test::black_box(SCATTERED.encode(&input));
	let mut output = Vec::with_capacity(input.len());

	b.bytes = input.len() as u64;
	b.iter(|| {
		output.clear();
		test::black_box(basenc::Encoding::decode_into(&SCATTERED, encoded.as_bytes(), &mut output).unwrap());
	});
}
