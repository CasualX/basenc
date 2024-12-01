/*! Optimized encoding routines for hex encoding.

Signature:

```ignore
/// bytes: Slice of input bytes
/// dest: Pointer to output bytes with available capacity of at least `len * 2`
/// base: Base character to use for encoding
pub unsafe fn encode(bytes: &[u8], mut dest: *mut u8, base: u8) -> *mut u8;
```
*/

mod scalar;

impl_arch_encode! {
	unsafe fn(bytes: &[u8], dest: *mut u8, base: u8) -> *mut u8;

	(any(target_arch = "x86_64", target_arch = "x86")) => {
		ssse3: "ssse3" is_x86_feature_detected!("ssse3");
		sse2: "sse2" is_x86_feature_detected!("sse2");
	},
}
