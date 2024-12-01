use super::*;

mod scalar;

impl_arch_decode! {
	unsafe fn(string: &[u8], base: &Base64, pad: Padding, dest: *mut u8) -> Result<*mut u8, crate::Error>;

	(any(target_arch = "x86_64", target_arch = "x86")) => {
		ssse3: "ssse3" is_x86_feature_detected!("ssse3");
		sse2: "sse2" is_x86_feature_detected!("sse2");
	},
}
