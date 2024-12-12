/*!
Incremental processing
======================

The [`RATIO`](Encoding::RATIO) associated constant provides information about the encoding ratio and can be used to compute the chunk size for incremental processing.

Encoding:

```
use std::mem;

fn encode<E: basenc::Encoding>(encoding: &E, bytes: &[u8]) {
	let mut stack_buf = mem::MaybeUninit::<[u8; 512]>::uninit();

	let chunk_size = E::RATIO.encoding_chunk_size(mem::size_of_val(&stack_buf));

	for chunk in bytes.chunks(chunk_size) {
		let string = encoding.encode_into(chunk, &mut stack_buf);
		// println!("{}", string);
	}
}
```

Decoding:

```
use std::mem;

fn decode<E: basenc::Encoding>(encoding: &E, string: &str) {
	let mut stack_buf = mem::MaybeUninit::<[u8; 512]>::uninit();

	let chunk_size = E::RATIO.decoding_chunk_size(mem::size_of_val(&stack_buf));

	for chunk in string.as_bytes().chunks(chunk_size) {
		let bytes = encoding.decode_into(chunk, &mut stack_buf).unwrap();
		// println!("{:x?}", bytes);
	}
}
```

*/

#[allow(unused_imports)]
use super::*;
