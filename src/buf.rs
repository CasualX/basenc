/*!
Abstracting over buffer types.
*/

use core::{mem, slice, str};

pub unsafe trait StackBuf {
	fn _as_ptr(&self) -> *const u8;
	fn _as_mut_ptr(&mut self) -> *mut u8;
	fn _len(&self) -> usize;
}

unsafe impl<const N: usize> StackBuf for mem::MaybeUninit<[u8; N]> {
	#[inline] fn _as_ptr(&self) -> *const u8 { (*self).as_ptr() as *const u8 }
	#[inline] fn _as_mut_ptr(&mut self) -> *mut u8 { (*self).as_mut_ptr() as *mut u8 }
	#[inline] fn _len(&self) -> usize { N }
}

unsafe impl<const N: usize> StackBuf for [u8; N] {
	#[inline] fn _as_ptr(&self) -> *const u8 { self as *const [u8; N] as *const u8 }
	#[inline] fn _as_mut_ptr(&mut self) -> *mut u8 { self as *mut [u8; N] as *mut u8 }
	#[inline] fn _len(&self) -> usize { N }
}

unsafe impl<const N: usize> StackBuf for [mem::MaybeUninit<u8>; N] {
	#[inline] fn _as_ptr(&self) -> *const u8 { self as *const _ as *const u8 }
	#[inline] fn _as_mut_ptr(&mut self) -> *mut u8 { self as *mut _ as *mut u8 }
	#[inline] fn _len(&self) -> usize { N }
}

unsafe impl StackBuf for [u8] {
	#[inline] fn _as_ptr(&self) -> *const u8 { (*self).as_ptr() }
	#[inline] fn _as_mut_ptr(&mut self) -> *mut u8 { (*self).as_mut_ptr() }
	#[inline] fn _len(&self) -> usize { (*self).len() }
}

unsafe impl StackBuf for [mem::MaybeUninit<u8>] {
	#[inline] fn _as_ptr(&self) -> *const u8 { (*self).as_ptr() as *const u8 }
	#[inline] fn _as_mut_ptr(&mut self) -> *mut u8 { (*self).as_mut_ptr() as *mut u8 }
	#[inline] fn _len(&self) -> usize { (*self).len() }
}

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
/// Convenience. Appends to the buffer and returns ownership.
/// - `Vec<u8>`
///
/// Efficient buffer reuse. Appends to the buffer.
/// - `&mut Vec<u8>`
///
/// Stack buffers. Panics if the buffer is too small.
/// - `&mut [u8]`
/// - `&mut [u8; N]`
/// - `&mut [MaybeUninit<u8>]`
/// - `&mut [MaybeUninit<u8>; N]`
/// - `&mut MaybeUninit<[u8; N]>`
pub trait DecodeBuf {
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
	unsafe fn allocate(&mut self, len: usize) -> *mut u8;

	/// Commits `len` bytes previously allocated.
	///
	/// Sets the buffer length effectively appending the written bytes to the output.
	///
	/// Returns the decoded bytes.
	///
	/// # Safety
	///
	/// * The length passed to `commit` must be less than or equal to the length passed to `allocate`.
	/// * No other access to the buffer may occur between `allocate` and `commit`.
	unsafe fn commit(self, len: usize) -> Self::Output;
}

impl<'a, T: StackBuf + ?Sized> DecodeBuf for &'a mut T {
	type Output = &'a [u8];
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > self._len() {
			buffer_too_small();
		}
		self._as_mut_ptr()
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= self._len());
		slice::from_raw_parts(self._as_ptr(), len)
	}
}

#[cfg(any(test, feature = "std"))]
impl DecodeBuf for ::std::vec::Vec<u8> {
	type Output = ::std::vec::Vec<u8>;
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

#[cfg(any(test, feature = "std"))]
impl<'a> DecodeBuf for &'a mut ::std::vec::Vec<u8> {
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
/// Convenience. Appends to the buffer and returns ownership.
/// - `String`
///
/// Efficient buffer reuse. Appends to the buffer.
/// - `&mut String`
/// - `&mut Vec<u8>`
///
/// Stack buffers. Panics if the buffer is too small.
/// - `&mut [u8]`
/// - `&mut [u8; N]`
/// - `&mut [MaybeUninit<u8>]`
/// - `&mut [MaybeUninit<u8>; N]`
/// - `&mut MaybeUninit<[u8; N]>`
pub trait EncodeBuf {
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
	unsafe fn allocate(&mut self, len: usize) -> *mut u8;

	/// Commits `len` bytes previously allocated.
	///
	/// Sets the buffer length effectively appending the written bytes to the output.
	///
	/// Returns the encoded string.
	///
	/// # Safety
	///
	/// * The length passed to `commit` must be less than or equal to the length passed to `allocate`.
	/// * The caller must write only valid UTF-8 to the returned memory.
	/// * No other access to the buffer may occur between `allocate` and `commit`.
	unsafe fn commit(self, len: usize) -> Self::Output;
}

impl<'a, T: StackBuf + ?Sized> EncodeBuf for &'a mut T {
	type Output = &'a str;
	unsafe fn allocate(&mut self, len: usize) -> *mut u8 {
		if len > self._len() {
			buffer_too_small();
		}
		self._as_mut_ptr()
	}
	unsafe fn commit(self, len: usize) -> Self::Output {
		debug_assert!(len <= self._len());
		str::from_utf8_unchecked(slice::from_raw_parts(self._as_ptr(), len))
	}
}

#[cfg(any(test, feature = "std"))]
impl EncodeBuf for ::std::string::String {
	type Output = ::std::string::String;
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

#[cfg(any(test, feature = "std"))]
impl<'a> EncodeBuf for &'a mut ::std::string::String {
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

#[cfg(any(test, feature = "std"))]
impl<'a> EncodeBuf for &'a mut ::std::vec::Vec<u8> {
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
