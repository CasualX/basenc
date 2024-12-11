/*!
Abstracting over buffer types.
*/

use core::{mem, slice, str};

//----------------------------------------------------------------

/// Byte buffer receiving decoded input.
///
/// # Implementors
///
/// * `&mut MaybeUninit<[u8; N]>`, `&mut [u8; N]`, `&mut [u8]`: Stack buffers. Panics if the buffer is too small.
///
/// * `Vec<u8>`: Convenience. Appends to the buffer.
///
/// * `&mut Vec<u8>`: Efficient buffer reuse. Appends to the buffer.
///
/// # Examples
///
/// Start by calculating the `upper_bound` of memory needed for decoding and `buffer.allocate(upper_bound)` it.
///
/// Write at most `upper_bound` of decoded bytes to this memory and `buffer.commit(len)` where `len` is the actual number of bytes written.
pub trait DecodeBuf {
	type Output;

	/// Returns uninitialized memory of the requested length.
	///
	/// Increases the underlying buffer's capacity and returns those extra bytes without touching the buffer length.
	///
	/// # Safety
	///
	/// The returned memory is logically uninitialized.
	unsafe fn allocate(&mut self, len: usize) -> *mut u8;

	/// Commits `len` bytes previously allocated.
	///
	/// Sets the buffer length effectively appending the written bytes to the output.
	///
	/// Returns the decoded bytes.
	///
	/// # Safety
	///
	/// The committed `len` must be less than or equal to the earlier allocated `len`.
	///
	/// The buffer must not be touched in between calling `allocate` and `commit`.
	unsafe fn commit(self, len: usize) -> Self::Output;
}

impl<'a, const N: usize> DecodeBuf for &'a mut mem::MaybeUninit<[u8; N]> {
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

impl<'a, const N: usize> DecodeBuf for &'a mut [mem::MaybeUninit<u8>; N] {
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

impl<'a, const N: usize> DecodeBuf for &'a mut [u8; N] {
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

impl<'a> DecodeBuf for &'a mut [mem::MaybeUninit<u8>] {
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

impl<'a> DecodeBuf for &'a mut [u8] {
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

#[cfg(any(test, feature = "std"))]
impl DecodeBuf for ::std::vec::Vec<u8> {
	type Output = ::std::vec::Vec<u8>;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		self.reserve(len);
		self.as_mut_ptr().offset(self.len() as isize)
	}
	unsafe fn commit(mut self, len: usize) -> Self::Output {
		let new_len = self.len() + len;
		self.set_len(new_len);
		self
	}
}

#[cfg(any(test, feature = "std"))]
impl<'a> DecodeBuf for &'a mut ::std::vec::Vec<u8> {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		self.reserve(len);
		self.as_mut_ptr().offset(self.len() as isize)
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		let start = self.len();
		self.set_len(start + len);
		slice::from_raw_parts(self.as_ptr().offset(start as isize), len)
	}
}

//----------------------------------------------------------------

/// String buffer receiving encoded input.
///
/// # Implementors
///
/// * `&mut MaybeUninit<[u8; N]>`, `&mut [u8; N]`, `&mut [u8]`: Stack buffers. Panics if the buffer is too small.
///
/// * `String`: Convenience. Appends to the buffer.
///
/// * `&mut String`, `&mut Vec<u8>`: Efficient buffer reuse. Appends to the buffer.
///
/// # Examples
///
/// Start by calculating the `upper_bound` of memory needed for encoding and `buffer.allocate(upper_bound)` it.
///
/// Write at most `upper_bound` of valid utf-8 bytes to this memory and `buffer.commit(len)` where `len` is the actual number of utf-8 bytes written.
pub trait EncodeBuf {
	type Output;

	/// Returns uninitialized memory of the requested length.
	///
	/// Increases the underlying buffer's capacity and returns those extra bytes without touching the buffer length.
	///
	/// # Safety
	///
	/// The returned memory is logically uninitialized.
	unsafe fn allocate(&mut self, len: usize) -> *mut u8;

	/// Commits `len` bytes previously allocated.
	///
	/// Sets the buffer length effectively appending the written bytes to the output.
	///
	/// Returns the encoded string.
	///
	/// # Safety
	///
	/// The committed `len` must be less than or equal to the earlier allocated `len`.
	///
	/// The bytes written must be valid utf-8.
	///
	/// The buffer must not be touched in between calling `allocate` and `commit`.
	unsafe fn commit(self, len: usize) -> Self::Output;
}

impl<'a, const N: usize> EncodeBuf for &'a mut mem::MaybeUninit<[u8; N]> {
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

impl<'a, const N: usize> EncodeBuf for &'a mut [mem::MaybeUninit<u8>; N] {
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

impl<'a, const N: usize> EncodeBuf for &'a mut [u8; N] {
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

impl<'a> EncodeBuf for &'a mut [mem::MaybeUninit<u8>] {
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

impl<'a> EncodeBuf for &'a mut [u8] {
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

#[cfg(any(test, feature = "std"))]
impl EncodeBuf for ::std::string::String {
	type Output = ::std::string::String;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		let vec = self.as_mut_vec();
		vec.reserve(len);
		vec.as_mut_ptr().offset(vec.len() as isize)
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

#[cfg(any(test, feature = "std"))]
impl<'a> EncodeBuf for &'a mut ::std::string::String {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		let vec = self.as_mut_vec();
		vec.reserve(len);
		vec.as_mut_ptr().offset(vec.len() as isize)
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		let vec = self.as_mut_vec();
		let start = vec.len();
		vec.set_len(start + len);
		let bytes = slice::from_raw_parts(vec.as_ptr().offset(start as isize), len);
		str::from_utf8_unchecked(bytes)
	}
}

#[cfg(any(test, feature = "std"))]
impl<'a> EncodeBuf for &'a mut ::std::vec::Vec<u8> {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		self.reserve(len);
		self.as_mut_ptr().offset(self.len() as isize)
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		let start = self.len();
		self.set_len(start + len);
		let bytes = slice::from_raw_parts(self.as_ptr().offset(start as isize), len);
		str::from_utf8_unchecked(bytes)
	}
}

#[cold]
const fn buffer_too_small() {
	panic!("buffer too small");
}
