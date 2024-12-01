/*! Optimized decoding routines for hex encoding.

Signature:

```ignore
/// string: Slice of input bytes
/// dest: Pointer to output bytes
pub unsafe fn decode(string: &[u8], dest: *mut u8) -> Result<*mut u8, crate::Error>;
```
*/

mod scalar;

impl_arch_decode! {
	unsafe fn(string: &[u8], dest: *mut u8) -> Result<*mut u8, crate::Error>;

	(any(target_arch = "x86_64", target_arch = "x86")) => {
		sse2: "sse2" is_x86_feature_detected!("sse2");
	},
}
