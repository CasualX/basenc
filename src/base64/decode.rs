use super::*;

mod scalar;

impl_arch_decode! {
	unsafe fn(string: &[u8], base: &Base64, pad: Padding, dest: *mut u8) -> Result<*mut u8, crate::Error>;

	(any(target_arch = "x86_64", target_arch = "x86")) => {
		avx2: "avx2" is_x86_feature_detected!("avx2");
		ssse3: "ssse3" is_x86_feature_detected!("ssse3");
		sse2: "sse2" is_x86_feature_detected!("sse2");
	},
	(all(target_arch = "aarch64", target_endian = "little")) => {
		neon: "neon" std::arch::is_aarch64_feature_detected!("neon");
	},
	(all(target_arch = "wasm32", target_feature = "simd128")) => {
		simd128: "simd128" cfg!(target_feature = "simd128");
	},
}
