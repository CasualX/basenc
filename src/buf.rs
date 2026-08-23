/*!
Abstracting over buffer types.
*/

use core::{mem, slice, str};

//----------------------------------------------------------------

/// Byte buffer receiving decoded input.
///
/// # Usage
///
/// Calculate the `upper_bound` of memory needed for decoding and pass it with `buffer.allocate(upper_bound)`.
///
/// This returns a pointer to uninitialized memory of the requested length. May panic if the buffer is too small.
///
/// Write at most `upper_bound` of decoded bytes to this memory and invoke `buffer.commit(len)` where `len` is the actual number of bytes written.
///
/// # Implementors
///
/// With the `alloc` feature:
///
/// Convenience. Appends to the buffer and returns ownership.
/// - `Vec<u8>`
///
/// Efficient buffer reuse. Appends to the buffer.
/// - `&mut Vec<u8>`
///
/// Available in all configurations:
///
/// Stack buffers. Panics if the buffer is too small.
/// - `&mut [u8]`
/// - `&mut [u8; N]`
/// - `&mut [MaybeUninit<u8>]`
/// - `&mut [MaybeUninit<u8>; N]`
/// - `&mut MaybeUninit<[u8; N]>`
///
/// # Safety
///
/// Implementors must uphold the contracts documented on [`allocate`](DecodeBuf::allocate) and [`commit`](DecodeBuf::commit).
pub unsafe trait DecodeBuf {
	type Output;

	/// Returns a non-null pointer to uninitialized memory valid for writes up to `len` bytes.
	///
	/// Increases the underlying buffer's capacity and returns those extra bytes without touching the buffer length.
	///
	/// # Safety
	///
	/// * The returned pointer from `allocate(len)` must be non-null, and valid for writes of exactly `len` bytes.
	/// * The allocated memory is logically uninitialized and must not be read before being written.
	/// * The memory must remain valid until `commit` is called.
	/// * No other access to the buffer may occur between `allocate` and `commit`.
	/// * Dropping the buffer without calling `commit` is sound.
	unsafe fn allocate(&mut self, len: usize) -> *mut u8;

	/// Commits `len` bytes previously allocated.
	///
	/// Sets the buffer length effectively appending the written bytes to the output.
	///
	/// Returns the decoded bytes.
	///
	/// # Safety
	///
	/// * This must follow a successful call to `allocate` on the same buffer.
	/// * The length passed to `commit` must be less than or equal to the length passed to `allocate`.
	/// * The first `len` bytes of the allocated memory must have been initialized.
	/// * No other access to the buffer may occur between `allocate` and `commit`.
	unsafe fn commit(self, len: usize) -> Self::Output;
}

unsafe impl<'a, const N: usize> DecodeBuf for &'a mut mem::MaybeUninit<[u8; N]> {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > N {
			buffer_too_small();
		}
		self.as_mut_ptr() as *mut u8
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= N);
		slice::from_raw_parts(self.as_ptr() as *const u8, len)
	}
}

unsafe impl<'a, const N: usize> DecodeBuf for &'a mut [mem::MaybeUninit<u8>; N] {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > N {
			buffer_too_small();
		}
		self.as_mut_ptr() as *mut u8
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= N);
		slice::from_raw_parts(self.as_ptr() as *const u8, len)
	}
}

unsafe impl<'a, const N: usize> DecodeBuf for &'a mut [u8; N] {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > N {
			buffer_too_small();
		}
		self.as_mut_ptr()
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= N);
		slice::from_raw_parts(self.as_ptr(), len)
	}
}

unsafe impl<'a> DecodeBuf for &'a mut [mem::MaybeUninit<u8>] {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > self.len() {
			buffer_too_small();
		}
		self.as_mut_ptr() as *mut u8
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= self.len());
		slice::from_raw_parts(self.as_ptr() as *const u8, len)
	}
}

unsafe impl<'a> DecodeBuf for &'a mut [u8] {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > self.len() {
			buffer_too_small();
		}
		self.as_mut_ptr()
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= self.len());
		slice::from_raw_parts(self.as_ptr(), len)
	}
}

#[cfg(feature = "alloc")]
unsafe impl DecodeBuf for alloc::vec::Vec<u8> {
	type Output = alloc::vec::Vec<u8>;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		self.reserve(len);
		self.as_mut_ptr().add(self.len())
	}
	unsafe fn commit(mut self, len: usize) -> Self::Output {
		let new_len = self.len() + len;
		self.set_len(new_len);
		self
	}
}

#[cfg(feature = "alloc")]
unsafe impl<'a> DecodeBuf for &'a mut alloc::vec::Vec<u8> {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		self.reserve(len);
		self.as_mut_ptr().add(self.len())
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		let start = self.len();
		self.set_len(start + len);
		slice::from_raw_parts(self.as_ptr().add(start), len)
	}
}

//----------------------------------------------------------------

/// String buffer receiving encoded input.
///
/// # Usage
///
/// Calculate the `upper_bound` of memory needed for encoding and pass it with `buffer.allocate(upper_bound)`.
///
/// This returns a pointer to uninitialized memory of the requested length. May panic if the buffer is too small.
///
/// Write at most `upper_bound` of valid UTF-8 bytes to this memory and invoke `buffer.commit(len)` where `len` is the actual number of UTF-8 bytes written.
///
/// # Implementors
///
/// With the `alloc` feature:
///
/// Convenience. Appends to the buffer and returns ownership.
/// - `String`
///
/// Efficient buffer reuse. Appends to the buffer.
/// - `&mut String`
/// - `&mut Vec<u8>`
///
/// Available in all configurations:
///
/// Stack buffers. Panics if the buffer is too small.
/// - `&mut [u8]`
/// - `&mut [u8; N]`
/// - `&mut [MaybeUninit<u8>]`
/// - `&mut [MaybeUninit<u8>; N]`
/// - `&mut MaybeUninit<[u8; N]>`
///
/// # Safety
///
/// Implementors must uphold the contracts documented on [`allocate`](EncodeBuf::allocate) and [`commit`](EncodeBuf::commit).
pub unsafe trait EncodeBuf {
	type Output;

	/// Returns a non-null pointer to uninitialized memory valid for writes up to `len` bytes.
	///
	/// Increases the underlying buffer's capacity and returns those extra bytes without touching the buffer length.
	///
	/// # Safety
	///
	/// * The returned pointer from `allocate(len)` must be non-null, and valid for writes of exactly `len` bytes.
	/// * The allocated memory is logically uninitialized and must not be read before being written.
	/// * The memory must remain valid until `commit` is called.
	/// * No other access to the buffer may occur between `allocate` and `commit`.
	/// * Dropping the buffer without calling `commit` is sound.
	unsafe fn allocate(&mut self, len: usize) -> *mut u8;

	/// Commits `len` bytes previously allocated.
	///
	/// Sets the buffer length effectively appending the written bytes to the output.
	///
	/// Returns the encoded string.
	///
	/// # Safety
	///
	/// * This must follow a successful call to `allocate` on the same buffer.
	/// * The length passed to `commit` must be less than or equal to the length passed to `allocate`.
	/// * The first `len` bytes of the allocated memory must have been initialized and contain valid UTF-8.
	/// * No other access to the buffer may occur between `allocate` and `commit`.
	unsafe fn commit(self, len: usize) -> Self::Output;
}

unsafe impl<'a, const N: usize> EncodeBuf for &'a mut mem::MaybeUninit<[u8; N]> {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > N {
			buffer_too_small();
		}
		self.as_mut_ptr() as *mut u8
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= N);
		let bytes = slice::from_raw_parts(self.as_ptr() as *const u8, len);
		str::from_utf8_unchecked(bytes)
	}
}

unsafe impl<'a, const N: usize> EncodeBuf for &'a mut [mem::MaybeUninit<u8>; N] {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > N {
			buffer_too_small();
		}
		self.as_mut_ptr() as *mut u8
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= N);
		let bytes = slice::from_raw_parts(self.as_ptr() as *const u8, len);
		str::from_utf8_unchecked(bytes)
	}
}

unsafe impl<'a, const N: usize> EncodeBuf for &'a mut [u8; N] {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > N {
			buffer_too_small();
		}
		self.as_mut_ptr()
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= N);
		let bytes = slice::from_raw_parts(self.as_ptr(), len);
		str::from_utf8_unchecked(bytes)
	}
}

unsafe impl<'a> EncodeBuf for &'a mut [mem::MaybeUninit<u8>] {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > self.len() {
			buffer_too_small();
		}
		self.as_mut_ptr() as *mut u8
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= self.len());
		let bytes = slice::from_raw_parts(self.as_ptr() as *const u8, len);
		str::from_utf8_unchecked(bytes)
	}
}

unsafe impl<'a> EncodeBuf for &'a mut [u8] {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > self.len() {
			buffer_too_small();
		}
		self.as_mut_ptr()
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= self.len());
		let bytes = slice::from_raw_parts(self.as_ptr(), len);
		str::from_utf8_unchecked(bytes)
	}
}

#[cfg(feature = "alloc")]
unsafe impl EncodeBuf for alloc::string::String {
	type Output = alloc::string::String;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		let vec = self.as_mut_vec();
		vec.reserve(len);
		vec.as_mut_ptr().add(vec.len())
	}
	unsafe fn commit(mut self, len: usize) -> Self::Output {
		{
			let vec = self.as_mut_vec();
			let new_len = vec.len() + len;
			vec.set_len(new_len);
		}
		self
	}
}

#[cfg(feature = "alloc")]
unsafe impl<'a> EncodeBuf for &'a mut alloc::string::String {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		let vec = self.as_mut_vec();
		vec.reserve(len);
		vec.as_mut_ptr().add(vec.len())
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		let vec = self.as_mut_vec();
		let start = vec.len();
		vec.set_len(start + len);
		let bytes = slice::from_raw_parts(vec.as_ptr().add(start), len);
		str::from_utf8_unchecked(bytes)
	}
}

#[cfg(feature = "alloc")]
unsafe impl<'a> EncodeBuf for &'a mut alloc::vec::Vec<u8> {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		self.reserve(len);
		self.as_mut_ptr().add(self.len())
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		let start = self.len();
		self.set_len(start + len);
		let bytes = slice::from_raw_parts(self.as_ptr().add(start), len);
		str::from_utf8_unchecked(bytes)
	}
}

#[cold]
const fn buffer_too_small() {
	panic!("buffer too small");
}
