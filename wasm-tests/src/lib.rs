#![no_std]

use basenc::{Base64, Base64Std, Base64Url, LowerHex, Padding, UpperHex};

const CHECKS: u32 = 0x03ff;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
	core::arch::wasm32::unreachable()
}

fn input() -> [u8; 4096] {
	let mut bytes = [0u8; 4096];
	let mut state = 0x1234_5678u32;
	let mut index = 0;
	while index < bytes.len() {
		state ^= state << 13;
		state ^= state >> 17;
		state ^= state << 5;
		bytes[index] = state as u8;
		index += 1;
	}
	bytes
}

fn exhaustive_validation() -> bool {
	let std = Base64Std.pad(Padding::Forbidden);
	let url = Base64Url.pad(Padding::Forbidden);
	let mut output = [0u8; 32];
	for byte in 0..=u8::MAX {
		let mut string = [b'A'; 32];
		string[8] = byte;
		let common = byte.is_ascii_uppercase() || byte.is_ascii_lowercase() || byte.is_ascii_digit();
		if std.decode_bytes_into(&string, &mut output).is_ok() != (common || byte == b'+' || byte == b'/') {
			return false;
		}
		if url.decode_bytes_into(&string, &mut output).is_ok() != (common || byte == b'-' || byte == b'_') {
			return false;
		}

		let mut hex = [b'0'; 32];
		hex[8] = byte;
		if LowerHex.decode_bytes_into(&hex, &mut output).is_ok() != byte.is_ascii_hexdigit() {
			return false;
		}
	}
	true
}

fn stores_in_bounds() -> bool {
	const CANARY: u8 = 0xa5;
	let base64 = Base64Std.pad(Padding::Forbidden);
	let mut input = [0u8; 256];
	let mut index = 0;
	while index < input.len() {
		input[index] = (index as u8).wrapping_mul(37);
		index += 1;
	}

	for len in 0..=input.len() {
		let encoded_capacity = len.saturating_add(2) / 3 * 4;
		let mut encoded_storage = [CANARY; 520];
		let encoded = base64.encode_into(&input[..len], &mut encoded_storage[..encoded_capacity]);
		let decoded_capacity = encoded.len().saturating_add(3) / 4 * 3;
		let mut decoded_storage = [CANARY; 260];
		let Ok(decoded) = base64.decode_into(encoded, &mut decoded_storage[..decoded_capacity])
		else {
			return false;
		};
		if decoded != &input[..len]
			|| encoded_storage[encoded_capacity..].iter().any(|&byte| byte != CANARY)
			|| decoded_storage[decoded_capacity..].iter().any(|&byte| byte != CANARY)
		{
			return false;
		}

		let encoded_capacity = len * 2;
		let mut encoded_storage = [CANARY; 520];
		let encoded = LowerHex.encode_into(&input[..len], &mut encoded_storage[..encoded_capacity]);
		let mut decoded_storage = [CANARY; 260];
		let Ok(decoded) = LowerHex.decode_into(encoded, &mut decoded_storage[..len])
		else {
			return false;
		};
		if decoded != &input[..len]
			|| encoded_storage[encoded_capacity..].iter().any(|&byte| byte != CANARY)
			|| decoded_storage[len..].iter().any(|&byte| byte != CANARY)
		{
			return false;
		}
	}
	true
}

#[unsafe(no_mangle)]
pub extern "C" fn expected_checks() -> u32 {
	CHECKS
}

#[unsafe(no_mangle)]
pub extern "C" fn verify() -> u32 {
	let bytes = input();
	let mut encoded = [0u8; 8192];
	let mut decoded = [0u8; 8192];
	let mut checks = 0;

	let value = Base64Std.encode_into(&bytes, &mut encoded);
	if Base64Std.decode_into(value, &mut decoded).ok() == Some(bytes.as_slice()) { checks |= 1 << 0; }
	let value = Base64Url.encode_into(&bytes, &mut encoded);
	if Base64Url.decode_into(value, &mut decoded).ok() == Some(bytes.as_slice()) { checks |= 1 << 1; }
	let custom_base = Base64::new(b'!', b'~');
	let custom = custom_base.pad(Padding::Forbidden);
	let value = custom.encode_into(&bytes, &mut encoded);
	if custom.decode_into(value, &mut decoded).ok() == Some(bytes.as_slice()) { checks |= 1 << 2; }
	let value = LowerHex.encode_into(&bytes, &mut encoded);
	if LowerHex.decode_into(value, &mut decoded).ok() == Some(bytes.as_slice()) { checks |= 1 << 3; }
	let value = UpperHex.encode_into(&bytes, &mut encoded);
	if UpperHex.decode_into(value, &mut decoded).ok() == Some(bytes.as_slice()) { checks |= 1 << 4; }

	let bad64 = b"QUJDREVGR0hJSkt!TU5PUFFSU1RVVldY";
	if Base64Std.decode_bytes_into(bad64, &mut decoded).err().map(|error| error.offset) == Some(15) { checks |= 1 << 5; }
	let bad_hex = b"00112233445566778899aabbccddeef!";
	if LowerHex.decode_bytes_into(bad_hex, &mut decoded).err().map(|error| error.offset) == Some(31) { checks |= 1 << 6; }
	if exhaustive_validation() { checks |= 1 << 7; }
	if stores_in_bounds() { checks |= 1 << 8; }

	let generic = Base64::new(b'+', b'/');
	let dedicated_value = Base64Std.encode_into(&bytes, &mut encoded);
	let mut generic_encoded = [0u8; 8192];
	let generic_value = generic.encode_into(&bytes, &mut generic_encoded);
	if dedicated_value == generic_value { checks |= 1 << 9; }

	checks
}

fn hash(mut state: u32, bytes: &[u8]) -> u32 {
	for &byte in bytes {
		state ^= byte as u32;
		state = state.wrapping_mul(0x0100_0193);
	}
	state
}

#[unsafe(no_mangle)]
pub extern "C" fn fingerprint() -> u32 {
	let bytes = input();
	let mut output = [0u8; 8192];
	let mut state = 0x811c_9dc5;
	state = hash(state, Base64Std.encode_into(&bytes, &mut output).as_bytes());
	state = hash(state, Base64Url.encode_into(&bytes, &mut output).as_bytes());
	let custom_base = Base64::new(b'!', b'~');
	state = hash(state, custom_base.pad(Padding::Forbidden).encode_into(&bytes, &mut output).as_bytes());
	state = hash(state, LowerHex.encode_into(&bytes, &mut output).as_bytes());
	hash(state, UpperHex.encode_into(&bytes, &mut output).as_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_base64_encode(iterations: u32) -> u32 {
	let bytes = input();
	let mut encoded = [0u8; 8192];
	let mut checksum = 0u32;
	let mut index = 0;
	while index < iterations {
		let value = Base64Std.encode_into(&bytes, &mut encoded);
		checksum = checksum.wrapping_add(value.as_bytes()[index as usize % value.len()] as u32);
		index += 1;
	}
	checksum
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_base64_decode(iterations: u32) -> u32 {
	let bytes = input();
	let mut encoded = [0u8; 8192];
	let value = Base64Std.encode_into(&bytes, &mut encoded);
	let mut decoded = [0u8; 8192];
	let mut checksum = 0u32;
	let mut index = 0;
	while index < iterations {
		let value = Base64Std.decode_into(value, &mut decoded).unwrap();
		checksum = checksum.wrapping_add(value[index as usize % value.len()] as u32);
		index += 1;
	}
	checksum
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_generic_base64_encode(iterations: u32) -> u32 {
	let bytes = input();
	let base = Base64::new(b'+', b'/');
	let mut encoded = [0u8; 8192];
	let mut checksum = 0u32;
	let mut index = 0;
	while index < iterations {
		let value = base.encode_into(&bytes, &mut encoded);
		checksum = checksum.wrapping_add(value.as_bytes()[index as usize % value.len()] as u32);
		index += 1;
	}
	checksum
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_generic_base64_decode(iterations: u32) -> u32 {
	let bytes = input();
	let base = Base64::new(b'+', b'/');
	let mut encoded = [0u8; 8192];
	let value = base.encode_into(&bytes, &mut encoded);
	let mut decoded = [0u8; 8192];
	let mut checksum = 0u32;
	let mut index = 0;
	while index < iterations {
		let value = base.decode_into(value, &mut decoded).unwrap();
		checksum = checksum.wrapping_add(value[index as usize % value.len()] as u32);
		index += 1;
	}
	checksum
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_hex_encode(iterations: u32) -> u32 {
	let bytes = input();
	let mut encoded = [0u8; 8192];
	let mut checksum = 0u32;
	let mut index = 0;
	while index < iterations {
		let value = LowerHex.encode_into(&bytes, &mut encoded);
		checksum = checksum.wrapping_add(value.as_bytes()[index as usize % value.len()] as u32);
		index += 1;
	}
	checksum
}
