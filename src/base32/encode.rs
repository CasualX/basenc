use super::*;

mod scalar;

impl_arch_encode! {
	unsafe fn(bytes: &[u8], base: &Base32, pad: Padding, dest: *mut u8) -> *mut u8;

	(any(target_arch = "x86_64", target_arch = "x86")) => {
		avx2: "avx2" is_x86_feature_detected!("avx2");
		ssse3: "ssse3" is_x86_feature_detected!("ssse3");
	},
}
