/*! Optimized encoding routines for hex encoding.

Signature:

```ignore
/// src: Pointer to input bytes
/// len: Number of input bytes
/// dest: Pointer to output bytes with available capacity of at least `len * 2`
/// base: Base character to use for encoding
pub unsafe fn encode(src: *const u8, len: usize, mut dest: *mut u8, base: u8);
```
*/

mod generic;

cfg_if::cfg_if! {
	if #[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), target_feature = "sse2"))] {
		mod sse2;
		pub use sse2::encode;
	}
	else {
		pub use generic::encode;
	}
}
