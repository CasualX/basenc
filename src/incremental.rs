/*!
Incremental processing
======================

The [`RATIO`](Encoding::RATIO) associated constant provides information about the encoding ratio and can be used to compute the chunk size for incremental processing.

Encoding:

```
use std::mem;

fn encode<E: basenc::Encoding>(encoding: &E, bytes: &[u8]) {
	let mut stack_buf = mem::MaybeUninit::<[u8; 512]>::uninit();
	let mut encoder = basenc::incremental::Encoder::new(encoding, bytes, &mut stack_buf);
	while let Some(chunk) = encoder.next() {
		// println!("{}", chunk);
	}
}
```

Decoding:

```
use std::mem;

fn decode<E: basenc::Encoding>(encoding: &E, string: &str) {
	let mut stack_buf = mem::MaybeUninit::<[u8; 512]>::uninit();
	let mut decoder = basenc::incremental::Decoder::new(encoding, string, &mut stack_buf);
	while let Some(chunk) = decoder.next() {
		// println!("{:x?}", result.unwrap());
	}
}
```

*/

use super::*;

/// An encoder that encodes data incrementally.
pub struct Encoder<'a, 'buf, E, B> {
	encoding: &'a E,
	chunks: slice::Chunks<'a, u8>,
	buffer: &'buf mut B,
}

impl<'a, 'buf, E: Encoding, B: StackBuf> Encoder<'a, 'buf, E, B> {
	#[inline]
	pub fn new(encoding: &'a E, bytes: &'a [u8], buffer: &'buf mut B) -> Encoder<'a, 'buf, E, B> {
		let chunk_size = E::RATIO.encoding_chunk_size(buffer._len());
		let chunks = bytes.chunks(chunk_size);
		Encoder { encoding, chunks, buffer }
	}
	#[inline]
	pub fn next(&'buf mut self) -> Option<&'buf str> {
		let chunk = self.chunks.next()?;
		Some(self.encoding.encode_into(chunk, &mut *self.buffer))
	}
}

/// A decoder that decodes data incrementally.
pub struct Decoder<'a, 'buf, E, B> {
	encoding: &'a E,
	chunks: slice::Chunks<'a, u8>,
	buffer: &'buf mut B,
}

impl<'a, 'buf, E: Encoding, B: StackBuf> Decoder<'a, 'buf, E, B> {
	#[inline]
	pub fn new(encoding: &'a E, string: &'a str, buffer: &'buf mut B) -> Decoder<'a, 'buf, E, B> {
		let chunk_size = E::RATIO.decoding_chunk_size(buffer._len());
		let chunks = string.as_bytes().chunks(chunk_size);
		Decoder { encoding, chunks, buffer }
	}
	#[inline]
	pub fn next(&'buf mut self) -> Option<Result<&'buf [u8], Error>> {
		let chunk = self.chunks.next()?;
		Some(self.encoding.decode_into(chunk, &mut *self.buffer))
	}
}
