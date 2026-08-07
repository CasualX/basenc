#![feature(test)]
extern crate test;

fn input(len: usize) -> Vec<u8> {
	(0..len).map(|index| index.wrapping_mul(37) as u8).collect()
}

macro_rules! hex_bench {
	($encode:ident, $decode:ident, $len:expr) => {
		#[bench]
		fn $encode(b: &mut test::Bencher) {
			let input = test::black_box(input($len));
			let mut output = basenc::LowerHex.encode(&input);
			output.clear();

			b.bytes = input.len() as u64;
			b.iter(|| {
				output.clear();
				test::black_box(basenc::LowerHex.encode_into(&input, &mut output));
			});
		}

		#[bench]
		fn $decode(b: &mut test::Bencher) {
			let input = input($len);
			let encoded = test::black_box(basenc::LowerHex.encode(&input));
			let mut output = basenc::LowerHex.decode(&encoded).unwrap();
			output.clear();

			// Report throughput in decoded bytes so encode and decode are comparable.
			b.bytes = input.len() as u64;
			b.iter(|| {
				output.clear();
				test::black_box(basenc::LowerHex.decode_into(&encoded, &mut output).unwrap());
			});
		}
	};
}

hex_bench!(encode_16, decode_16, 16);
hex_bench!(encode_32, decode_32, 32);
hex_bench!(encode_4k, decode_4k, 4 * 1024);
hex_bench!(encode_1m, decode_1m, 1024 * 1024);
