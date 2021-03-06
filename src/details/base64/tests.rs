use ::std::prelude::v1::*;
use super::*;
use super::{encode as encodev, decode as decodev};

fn roundtrip<C: CharSet>(bytes: &[u8], charset: C, string: &str) {
	assert_eq!(encodev(bytes, charset, String::new()), string);
	assert_eq!(decodev(string, charset, Vec::new()), Ok(bytes.to_owned()));
}

fn error<C: CharSet>(string: &str, charset: C, err: ::Error) {
	let mut buf = [0u8; 64];
	assert_eq!(decodev(string, charset, &mut buf[..]), Err(err));
}

#[test]
fn rfc4648() {
	// rfc4648 test vectors
	roundtrip(b"", Base64Std, "");
	roundtrip(b"f", Base64Std, "Zg==");
	roundtrip(b"fo", Base64Std, "Zm8=");
	roundtrip(b"foo", Base64Std, "Zm9v");
	roundtrip(b"foob", Base64Std, "Zm9vYg==");
	roundtrip(b"fooba", Base64Std, "Zm9vYmE=");
	roundtrip(b"foobar", Base64Std, "Zm9vYmFy");
}

#[test]
fn wikipedia() {
	// Padding test vectors from wikipedia: https://en.wikipedia.org/wiki/Base64
	roundtrip(b"any carnal pleasure.", Base64Std, "YW55IGNhcm5hbCBwbGVhc3VyZS4=");
	roundtrip(b"any carnal pleasure", Base64Std, "YW55IGNhcm5hbCBwbGVhc3VyZQ==");
	roundtrip(b"any carnal pleasur", Base64Std, "YW55IGNhcm5hbCBwbGVhc3Vy");
	roundtrip(b"any carnal pleasu", Base64Std, "YW55IGNhcm5hbCBwbGVhc3U=");
	roundtrip(b"any carnal pleas", Base64Std, "YW55IGNhcm5hbCBwbGVhcw==");
	roundtrip(b"pleasure.", Base64Std, "cGxlYXN1cmUu", );
	roundtrip(b"leasure.", Base64Std, "bGVhc3VyZS4=", );
	roundtrip(b"easure.", Base64Std, "ZWFzdXJlLg==", );
	roundtrip(b"asure.", Base64Std, "YXN1cmUu", );
	roundtrip(b"sure.", Base64Std, "c3VyZS4=", );
}
#[test]
fn cwgem_test_base64_rb() {
	// Some test vectors I found with google: https://gist.github.com/cwgem/1209735
	// Note: Those tests use '+' in the url safe alphabet!

	roundtrip(b"Send reinforcements", Base64Std, "U2VuZCByZWluZm9yY2VtZW50cw==");
	roundtrip(b"Now is the time for all good coders\nto learn Ruby", Base64Std,
		"Tm93IGlzIHRoZSB0aW1lIGZvciBhbGwgZ29vZCBjb2RlcnMKdG8gbGVhcm4gUnVieQ==");
	roundtrip(b"This is line one\nThis is line two\nThis is line three\nAnd so on...\n", Base64Std,
		"VGhpcyBpcyBsaW5lIG9uZQpUaGlzIGlzIGxpbmUgdHdvClRoaXMgaXMgbGluZSB0aHJlZQpBbmQgc28gb24uLi4K");
	roundtrip("テスト".as_bytes(), Base64Std, "44OG44K544OI");

	roundtrip(b"", Base64Std, "");
	roundtrip(b"\0", Base64Std, "AA==");
	roundtrip(b"\0\0", Base64Std, "AAA=");
	roundtrip(b"\0\0\0", Base64Std, "AAAA");
	roundtrip(b"\xFF", Base64Std, "/w==");
	roundtrip(b"\xFF\xFF", Base64Std, "//8=");
	roundtrip(b"\xFF\xFF\xFF", Base64Std, "////");
	roundtrip(b"\xff\xef", Base64Std, "/+8=");

	error("^", Base64Std, ::Error::BadLength);
	error("A", Base64Std, ::Error::BadLength);
	error("A^", Base64Std, ::Error::BadLength);
	error("AA", Base64Std, ::Error::BadLength);
	error("AA=", Base64Std, ::Error::BadLength);
	error("AA===", Base64Std, ::Error::BadLength);
	error("AA=x", Base64Std, ::Error::InvalidChar('x'));
	error("AAA", Base64Std, ::Error::BadLength);
	error("AAA^", Base64Std, ::Error::InvalidChar('^'));
	error("AB==", Base64Std, ::Error::Denormal);
	error("AAB=", Base64Std, ::Error::Denormal);
	
    roundtrip(b"", Base64Url, "");
    roundtrip(b"\0", Base64Url, "AA");
    roundtrip(b"\0\0", Base64Url, "AAA");
    roundtrip(b"\0\0\0", Base64Url, "AAAA");
    roundtrip(b"\xFF", Base64Url, "_w");
    roundtrip(b"\xFF\xFF", Base64Url, "__8");
    roundtrip(b"\xFF\xFF\xFF", Base64Url, "____");
	roundtrip(b"\xff\xef", Base64Url, "_-8");
}

// Build the lookup tables...
#[test]
fn build_luts() {
	print_lut("base64std", Base64Std.chars());
	print_lut("base64url", Base64Url.chars());
}
fn print_lut(name: &str, chars: &str) {
	// Build the LUT
	let mut table = [255u8; 256];
	for (index, byte) in chars.bytes().enumerate() {
		table[byte as usize] = index as u8;
	}
	// Find first non-zero byte
	let base = table[..].iter().enumerate().find(|&(_, &byte)| byte != 255).map(|(num, _)| num).unwrap();
	// Find last non-zero byte
	let end = table[..].iter().enumerate().rev().find(|&(_, &byte)| byte != 255).map(|(num, _)| num).unwrap();
	// Print it for manual adjustment
	println!("{}: ({}, &{:?})", name, base, &table[base..end + 1]);
}
