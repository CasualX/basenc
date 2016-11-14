use ::std::prelude::v1::*;
use super::*;

fn roundtrip<C: CharSet>(bytes: &[u8], charset: C, string: &str) {
	assert_eq!(encode(bytes, charset, String::new()), string);
	assert_eq!(decode(string, charset, Vec::new()), Ok(bytes.to_owned()));
}

#[test]
fn rfc4648() {
	// rfc4648 test vectors
	roundtrip(b"", UpperHex, "");
	roundtrip(b"f", UpperHex, "66");
	roundtrip(b"fo", UpperHex, "666F");
	roundtrip(b"foo", UpperHex, "666F6F");
	roundtrip(b"foob", UpperHex, "666F6F62");
	roundtrip(b"fooba", UpperHex, "666F6F6261");
	roundtrip(b"foobar", UpperHex, "666F6F626172");
}

#[test]
fn stuff() {
	let bytes = &[0x5a, 0xcf, 0xfd, 0xa7, 0xca, 0x3e, 0x37, 0x3d, 0x4a, 0x11][..];
	roundtrip(bytes, LowerHex, "5acffda7ca3e373d4a11");
	roundtrip(bytes, UpperHex, "5ACFFDA7CA3E373D4A11");
	assert_eq!(decode("5ACfFda7cA3e373D4a11", AnyHex, &mut [0u8; 16][..]), Ok(bytes));
	assert_eq!(decode("5acFfDA7Ca3E373d4A11", AnyHex, &mut [0u8; 16][..]), Ok(bytes));
}

#[test]
fn build_luts() {
	print_lut("lowerhex", LowerHex.chars(), "");
	print_lut("upperhex", UpperHex.chars(), "");
	print_lut("anyhex", LowerHex.chars(), "ABCDEF");
}
fn print_lut(name: &str, chars: &str, ci: &str) {
	// Build the LUT
	let mut table = [255u8; 256];
	for (index, byte) in chars.bytes().enumerate() {
		table[byte as usize] = index as u8;
	}
	for (index, byte) in ci.bytes().enumerate() {
		table[byte as usize] = index as u8 + 10;
	}
	// Find first non-zero byte
	let base = table[..].iter().enumerate().find(|&(_, &byte)| byte != 255).map(|(num, _)| num).unwrap();
	// Find last non-zero byte
	let end = table[..].iter().enumerate().rev().find(|&(_, &byte)| byte != 255).map(|(num, _)| num).unwrap();
	// Print it for manual adjustment
	println!("{}: ({}, &{:?})", name, base, &table[base..end + 1]);
}
