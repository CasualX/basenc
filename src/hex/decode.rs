/*! Optimized decoding routines for hex encoding.

Signature:

```ignore
/// src: Pointer to input bytes with at least `len * 2` bytes available
/// len: Number of output bytes
/// dest: Pointer to output bytes
pub unsafe fn decode(src: *const u8, len: usize, dest: *mut u8) -> Result<(), crate::Error>;
```
*/

mod generic;

cfg_if::cfg_if! {
	if #[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), target_feature = "sse2"))] {
		mod sse2;
		pub use sse2::decode;
	}
	// else if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
	// 	mod avx2;
	// 	pub use avx2::decode;
	// }
	else {
		pub use generic::decode;
	}
}
