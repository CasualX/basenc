// https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html

macro_rules! impl_arch_decode {
	(
		$signature:ty;
		$($target_arch:tt => {
			$($target_feature:ident: $target_feature_lit:literal $target_feature_detect:expr;)*
		},)*
	) => {
		cfg_select! {
			feature = "simd-off" => {
				#[inline]
				pub fn decode_fn() -> $signature {
					scalar::decode
				}
			}
			feature = "simd-runtime" => {
				static DECODE: core::sync::atomic::AtomicPtr<()> = core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

				#[inline]
				pub fn decode_fn() -> $signature {
					let ptr = DECODE.load(core::sync::atomic::Ordering::Relaxed);
					if !ptr.is_null() {
						return unsafe { core::mem::transmute(ptr) };
					}
					let decode = decode_detect();
					DECODE.store(decode as *mut (), core::sync::atomic::Ordering::Relaxed);
					decode
				}

				cfg_select! {
					$(all $target_arch => {
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
					_ => {
						#[inline]
						pub fn decode_detect() -> $signature {
							scalar::decode
						}
					}
				}

			}
			_ => {
				cfg_select! {
					$($(
						all(all $target_arch, target_feature = $target_feature_lit) => {
							mod $target_feature;

							#[inline]
							pub fn decode_fn() -> $signature {
								$target_feature::decode
							}
						}
					)*)*
					_ => {
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
		cfg_select! {
			feature = "simd-off" => {
				#[inline]
				pub fn encode_fn() -> $signature {
					scalar::encode
				}
			}
			feature = "simd-runtime" => {
				static ENCODE: core::sync::atomic::AtomicPtr<()> = core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

				#[inline]
				pub fn encode_fn() -> $signature {
					let ptr = ENCODE.load(core::sync::atomic::Ordering::Relaxed);
					if !ptr.is_null() {
						return unsafe { core::mem::transmute(ptr) };
					}
					let encode = encode_detect();
					ENCODE.store(encode as *mut (), core::sync::atomic::Ordering::Relaxed);
					encode
				}

				cfg_select! {
					$(all $target_arch => {
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
					_ => {
						#[inline]
						pub fn encode_detect() -> $signature {
							scalar::encode
						}
					}
				}

			}
			_ => {
				cfg_select! {
					$($(
						all(all $target_arch, target_feature = $target_feature_lit) => {
							mod $target_feature;

							#[inline]
							pub fn encode_fn() -> $signature {
								$target_feature::encode
							}
						}
					)*)*
					_ => {
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
