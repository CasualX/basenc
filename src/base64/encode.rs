use super::*;

mod scalar;

impl_arch_encode! {
	unsafe fn(bytes: &[u8], base: &Base64, pad: Padding, dest: *mut u8) -> *mut u8;

	(any(target_arch = "x86_64", target_arch = "x86")) => {
		ssse3: "ssse3" is_x86_feature_detected!("ssse3");
	},
}
