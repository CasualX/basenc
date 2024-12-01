// https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html

macro_rules! impl_arch_decode {
	(
		$signature:ty;
		$($target_arch:tt => {
			$($target_feature:ident: $target_feature_lit:literal $target_feature_detect:expr;)*
		},)*
	) => {
		cfg_if::cfg_if! {
			if #[cfg(feature = "simd-off")] {
				#[inline]
				pub fn decode_fn() -> $signature {
					scalar::decode
				}
			}
			else if #[cfg(feature = "simd-runtime")] {
				static mut DECODE: Option<$signature> = None;

				#[inline]
				pub fn decode_fn() -> $signature {
					unsafe {
						DECODE.unwrap_or_else(|| {
							let decode = decode_detect();
							DECODE = Some(decode);
							decode
						})
					}
				}

				cfg_if::cfg_if! {
					if #[cfg(any())] {}
					$(else if #[cfg $target_arch] {
						$(mod $target_feature;)*

						#[inline(never)]
						pub fn decode_detect() -> $signature {
							if false {unreachable!()}
							$(else if $target_feature_detect {
								return $target_feature::decode;
							})*
							else {
								return scalar::decode;
							}
						}
					})*
					else {
						#[inline]
						pub fn decode_detect() -> $signature {
							scalar::decode
						}
					}
				}

			}
			else {
				cfg_if::cfg_if! {
					if #[cfg(any())] {}
					$($(
						else if #[cfg(all(all $target_arch, target_feature = $target_feature_lit))] {
							mod $target_feature;

							#[inline]
							pub fn decode_fn() -> $signature {
								$target_feature::decode
							}
						}
					)*)*
					else {
						#[inline]
						pub fn decode_fn() -> $signature {
							scalar::decode
						}
					}
				}
			}
		}
	};
}

macro_rules! impl_arch_encode {
	(
		$signature:ty;
		$( $target_arch:tt => {
			$($target_feature:ident: $target_feature_lit:literal $target_feature_detect:expr;)*
		},)*
	) => {
		cfg_if::cfg_if! {
			if #[cfg(feature = "simd-off")] {
				#[inline]
				pub fn encode_fn() -> $signature {
					scalar::encode
				}
			}
			else if #[cfg(feature = "simd-runtime")] {
				static mut ENCODE: Option<$signature> = None;

				#[inline]
				pub fn encode_fn() -> $signature {
					unsafe {
						ENCODE.unwrap_or_else(|| {
							let encode = encode_detect();
							ENCODE = Some(encode);
							encode
						})
					}
				}

				cfg_if::cfg_if! {
					if #[cfg(any())] {}
					$(else if #[cfg $target_arch] {
						$(mod $target_feature;)*

						#[inline(never)]
						pub fn encode_detect() -> $signature {
							if false {unreachable!()}
							$(else if $target_feature_detect {
								return $target_feature::encode;
							})*
							else {
								return scalar::encode;
							}
						}
					})*
					else {
						#[inline]
						pub fn encode_detect() -> $signature {
							scalar::encode
						}
					}
				}

			}
			else {
				cfg_if::cfg_if! {
					if #[cfg(any())] {}
					$($(
						else if #[cfg(all(all $target_arch, target_feature = $target_feature_lit))] {
							mod $target_feature;

							#[inline]
							pub fn encode_fn() -> $signature {
								$target_feature::encode
							}
						}
					)*)*
					else {
						#[inline]
						pub fn encode_fn() -> $signature {
							scalar::encode
						}
					}
				}
			}
		}
	};
}
