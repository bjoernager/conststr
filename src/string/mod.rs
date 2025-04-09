// Copyright 2025 Gabriel Bjørnager Jensen.

mod test;

mod serde;

use crate::error::{LengthError, Utf8Error};

use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt::{self, Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::ptr::copy_nonoverlapping;
use core::slice::{self, SliceIndex};
use core::str::{self, FromStr};

#[cfg(feature = "alloc")]
use {
	alloc::borrow::Cow,
	alloc::boxed::Box,
};

#[cfg(feature = "oct")]
use {
	oct::decode::{self, Decode, DecodeBorrowed},
	oct::encode::{self, Encode, SizedEncode},
	oct::error::CollectionDecodeError,
};

#[cfg(feature = "std")]
use {
	std::ffi::OsStr,
	std::net::ToSocketAddrs,
	std::path::Path,
};

/// String container with maximum length.
///
/// This is in contrast to [`str`](prim@str) and the standard library's [`String`](alloc::string::String) type -- both of which have no size limit in practice.
///
/// The string itself is encoded in UTF-8 for interoperability wtih Rust's standard string facilities, and partly due to memory concerns.
/// Keep in mind that the size limit specified by `N` then denotes *octets* (or "bytes") and **not** *characters* -- i.e. a value of `8` may translate to anywhere between two and eight characters due to variable-length encoding.
///
/// # Examples
///
/// All instances of this type have the same size if the value of `N` is also the same.
/// Therefore, the following four strings have -- despite their different contents -- the same total size.
///
/// ```rust
/// use conststr::String;
/// use std::str::FromStr;
///
/// let s0 = String::<0x40>::default(); // Empty string.
/// let s1 = String::<0x40>::from_str("Hello there!").unwrap();
/// let s2 = String::<0x40>::from_str("أنا من أوروپا").unwrap();
/// let s3 = String::<0x40>::from_str("COGITO·ERGO·SUM").unwrap();
///
/// assert_eq!(size_of_val(&s0), size_of_val(&s1));
/// assert_eq!(size_of_val(&s0), size_of_val(&s2));
/// assert_eq!(size_of_val(&s0), size_of_val(&s3));
/// assert_eq!(size_of_val(&s1), size_of_val(&s2));
/// assert_eq!(size_of_val(&s1), size_of_val(&s3));
/// assert_eq!(size_of_val(&s2), size_of_val(&s3));
/// ```
#[derive(Clone, Copy)]
pub struct String<const N: usize> {
	len: usize,
	buf: [u8; N],
}

impl<const N: usize> String<N> {
	/// Constructs a new, constant string.
	///
	/// The provided string `s` is checked to be containable within `N` bytes.
	/// See also [`new_unchecked`](Self::new_unchecked).
	///
	/// # Errors
	///
	/// If the internal buffer cannot contain the entirety of `s`, then an error is returned.
	#[inline]
	#[track_caller]
	pub const fn new(s: &str) -> Result<Self, LengthError> {
		let len = s.len();

		if len > N {
			return Err(LengthError {
				remaining: N,
				count:     len,
			});
		}

		// SAFETY: We have tested that `s` is not too long.
		let this = unsafe { Self::new_unchecked(s) };
		Ok(this)
	}

	/// Unsafely constructs a new, constant string.
	///
	/// See also [`new`](Self::new) for a safe alternative to this constructor.
	///
	/// # Safety
	///
	/// If the internal buffer cannot contain the entirety of `s`, then the call to this constructor will result in undefined behaviour.
	#[inline]
	#[track_caller]
	pub const unsafe fn new_unchecked(s: &str) -> Self {
		let     len = s.len();
		let mut buf = [0x00; N];

		debug_assert!(len <= N, "cannot construct string from string slice that is longer");

		unsafe {
			let src = s.as_ptr();
			let dst = buf.as_mut_ptr();

			copy_nonoverlapping(src, dst, len);
		}

		// SAFETY: `str` can be assumed to only contain
		// valid UTF-8 data. The caller also guarantees
		// that `s` is not longer than `N`.
		unsafe { Self::from_raw_parts(buf, len) }
	}

	/// Constructs a new string from UTF-8 octets.
	///
	/// The passed slice is checked for its validity.
	/// For a similar function *without* these checks, see [`from_utf8_unchecked`](Self::from_utf8_unchecked).
	///
	/// # Errors
	///
	/// Each byte value must be a valid UTF-8 code point.
	/// If an invalid sequence is found, then this function will return an error.
	#[inline]
	#[track_caller]
	pub const fn from_utf8<const M: usize>(data: [u8; M]) -> Result<Self, Utf8Error> {
		if let Err(e) = str::from_utf8(&data) {
			let i = e.valid_up_to();
			let c = data[i];

			return Err(Utf8Error { value: c, index: i });
		}

		// SAFETY: `s` has been tested to only contain
		// valid octets.
		let this = unsafe { Self::from_utf8_unchecked(data) };
		Ok(this)
	}

	/// Unsafely constructs a new string from UTF-8 octets.
	///
	/// # Safety
	///
	/// Each byte value of the vector must be a valid UTF-8 code point.
	/// The behaviour of a programme that passes invalid values to this function is undefined.
	///
	/// Note that it is not undefined to call this function where `M` is greater than `N` as such usage will always be caught at compile-time.
	#[inline]
	#[must_use]
	#[track_caller]
	pub const unsafe fn from_utf8_unchecked<const M: usize>(data: [u8; M]) -> Self {
		const { assert!(M <= N, "cannot construct vector from array that is longer") };

		let len = M;

		let buf = if N == M {
			// Reuse the existing buffer.

			// SAFETY: We have tested that `N` and `M` are
			// equal.
			let ptr = &raw const data as *const [u8; N];

			unsafe { ptr.read() }
		} else {
			// Reallocate the buffer to `N` elements.

			let mut buf = [0x00; N];
			buf.copy_from_slice(&data);

			buf
		};

		// SAFETY: We have checked that the length is with-
		// in bounds.
		unsafe { Self::from_raw_parts(buf, len) }
	}

	/// Constructs a constant string from raw parts.
	///
	/// The provided parts are not tested in any way.
	///
	/// # Safety
	///
	/// The value of `len` may not exceed that of `N`.
	/// Additionally, the octets in `buf` (from index zero up to the value of `len - 1`) must be valid UTF-8 code points.
	///
	/// If any of these requirements are violated, behaviour is undefined.
	#[inline(always)]
	#[must_use]
	#[track_caller]
	pub const unsafe fn from_raw_parts(buf: [u8; N], len: usize) -> Self {
		debug_assert!(len <= N, "cannot construct string that is longer than its capacity");

		Self { len, buf }
	}

	/// Converts all ASCII characters to their uppercase equivalent.
	///
	/// Non-ASCII octets are ignored.
	#[inline(always)]
	pub const fn make_ascii_uppercase(&mut self) {
		self.as_mut_str().make_ascii_uppercase();
	}

	/// Converts all ASCII characters to their lowercase equivalent.
	///
	/// Non-ASCII octets are ignored.
	#[inline(always)]
	pub const fn make_ascii_lowercase(&mut self) {
		self.as_mut_str().make_ascii_lowercase();
	}

	/// Splits the string at an index, borrowing the two parts.
	///
	/// # Panics
	///
	/// If `mid` is not exactly on the boundary of a chacter (as per [`is_char_boundary`](Self::is_char_boundary)), then this method will panic.
	#[inline(always)]
	#[must_use]
	pub const fn split_at(&self, mid: usize) -> (&str, &str) {
		self.as_str().split_at(mid)
	}

	/// Splits the string at an index, borrowing the two parts.
	///
	/// If `mid` is not exactly on the boundary of a chacter (as per [`is_char_boundary`](Self::is_char_boundary)), then this method will instead return [`None`].
	#[inline(always)]
	#[must_use]
	pub const fn split_at_checked(&self, mid: usize) -> Option<(&str, &str)> {
		self.as_str().split_at_checked(mid)
	}

	/// Splits the string at an index, mutably borrowing the two parts.
	///
	/// # Panics
	///
	/// If `mid` is not exactly on the boundary of a chacter (as per [`is_char_boundary`](Self::is_char_boundary)), then this method will panic.
	#[inline(always)]
	#[must_use]
	pub const fn split_at_mut(&mut self, mid: usize) -> (&mut str, &mut str) {
		self.as_mut_str().split_at_mut(mid)
	}

	/// Splits the string at an index, mutably borrowing the two parts.
	///
	/// If `mid` is not exactly on the boundary of a chacter (as per [`is_char_boundary`](Self::is_char_boundary)), then this method will instead return [`None`].
	#[inline(always)]
	#[must_use]
	pub const fn split_at_mut_checked(&mut self, mid: usize) -> Option<(&mut str, &mut str)> {
		self.as_mut_str().split_at_mut_checked(mid)
	}

	/// Returns the current length of the string.
	///
	/// Remember that this value only denotes the octet count and **not** the amount of characters, graphemes, etc.
	#[inline(always)]
	#[must_use]
	pub const fn len(&self) -> usize {
		self.len
	}

	/// Checks if the string is empty, i.e. no characters are contained.
	#[inline(always)]
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.len() == 0x0
	}

	/// Checks if a specified index is on the boundary of a character.
	///
	/// In this case, character is defined as a set of one to four UTF-8 octets that represent a Unicode code point (specifically a Unicode scalar).
	#[inline(always)]
	#[must_use]
	pub const fn is_char_boundary(&self, index: usize) -> bool {
		self.as_str().is_char_boundary(index)
	}

	/// Checks if the entire string is also valid in ASCII.
	#[inline(always)]
	#[must_use]
	pub const fn is_ascii(&self) -> bool {
		self.as_str().is_ascii()
	}

	/// Gets a pointer to the first octet.
	#[inline(always)]
	#[must_use]
	pub const fn as_ptr(&self) -> *const u8 {
		self.buf.as_ptr()
	}

	/// Gets a mutable pointer to the first octet.
	///
	#[inline(always)]
	#[must_use]
	pub const fn as_mut_ptr(&mut self) -> *mut u8 {
		self.buf.as_mut_ptr()
	}

	/// Borrows the string as a byte slice.
	///
	/// The range of the returned slice only includes characters that are "used."
	#[inline(always)]
	#[must_use]
	pub const fn as_bytes(&self) -> &[u8] {
		// We need to use `from_raw_parts` to mark this
		// function `const`.

		let ptr = self.as_ptr();
		let len = self.len();

		unsafe { slice::from_raw_parts(ptr, len) }
	}

	/// Borrows the string as a mutable byte slice.
	///
	/// The range of the returned slice only includes characters that are "used."
	///
	/// # Safety
	///
	/// Writes to the returned slice mustn't contain invalid UTF-8 octets.
	#[inline(always)]
	#[must_use]
	pub const unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
		let ptr = self.as_mut_ptr();
		let len = self.len();

		unsafe { slice::from_raw_parts_mut(ptr, len) }
	}

	/// Borrows the string as a string slice.
	///
	/// The range of the returned slice only includes characters that are "used."
	#[inline(always)]
	#[must_use]
	pub const fn as_str(&self) -> &str {
		// SAFETY: We guarantee that all octets are always
		// valid UTF-8 code points.
		unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
	}

	/// Mutably borrows the string as a string slice.
	///
	/// The range of the returned slice only includes characters that are "used."
	#[inline(always)]
	#[must_use]
	pub const fn as_mut_str(&mut self) -> &mut str {
		unsafe {
			let ptr = self.as_mut_ptr();
			let len = self.len();

			let data = slice::from_raw_parts_mut(ptr, len);
			core::str::from_utf8_unchecked_mut(data)
		}
	}

	/// Destructs the provided string into its raw parts.
	///
	/// The returned parts are valid to pass back to [`from_raw_parts`](Self::from_raw_parts).
	///
	/// The returned byte array is guaranteed to be fully initialised.
	/// However, only octets up to an index of [`len`](Self::len) are also guaranteed to be valid UTF-8 code points.
	#[inline(always)]
	#[must_use]
	pub const fn into_raw_parts(self) -> ([u8; N], usize) {
		let Self { buf, len } = self;
		(buf, len)
	}

	/// Converts the constant string into a boxed string slice.
	#[cfg(feature = "alloc")]
	#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
	#[inline(always)]
	#[must_use]
	pub fn into_boxed_str(self) -> Box<str> {
		self.as_str().into()
	}

	/// Converts the constant string into a dynamic string.
	///
	/// The capacity of the resulting [`alloc::string::String`] object is equal to the value of `N`.
	#[cfg(feature = "alloc")]
	#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
	#[inline(always)]
	#[must_use]
	pub fn into_std_string(self) -> alloc::string::String {
		self.as_str().into()
	}
}

impl<const N: usize> AsMut<str> for String<N> {
	#[inline(always)]
	fn as_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
impl<const N: usize> AsRef<OsStr> for String<N> {
	#[inline(always)]
	fn as_ref(&self) -> &OsStr {
		self.as_str().as_ref()
	}
}

#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
impl<const N: usize> AsRef<Path> for String<N> {
	#[inline(always)]
	fn as_ref(&self) -> &Path {
		self.as_str().as_ref()
	}
}

impl<const N: usize> AsRef<str> for String<N> {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<const N: usize> AsRef<[u8]> for String<N> {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<const N: usize> Borrow<str> for String<N> {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<const N: usize> BorrowMut<str> for String<N> {
	#[inline(always)]
	fn borrow_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

impl<const N: usize> Debug for String<N> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.as_str(), f)
	}
}

#[cfg(feature = "oct")]
#[cfg_attr(doc, doc(cfg(feature = "oct")))]
impl<const N: usize> Decode for String<N> {
	type Error = CollectionDecodeError<LengthError, Utf8Error>;

	#[inline]
	#[track_caller]
	fn decode(input: &mut decode::Input) -> Result<Self, Self::Error> {
		let Ok(len) = Decode::decode(input);

		if len > N {
			return Err(CollectionDecodeError::BadLength(
				LengthError {
					remaining: N,
					count:     len,
				}
			));
		}

		let mut buf = [0x00; N];
		input.read_into(&mut buf[..len]);

		if let Err(e) = str::from_utf8(&buf[..len]) {
			let i = e.valid_up_to();
			let c = buf[i];

			return Err(CollectionDecodeError::BadItem(
				Utf8Error {
					value: c,
					index: i,
				},
			));
		}

		// SAFETY: We have tested that the first `len`
		// octets are valid UTF-8. The remaining, if any,
		// are null and therefore also valid.
		let this = unsafe { Self::from_raw_parts(buf, len) };
		Ok(this)
	}
}

#[cfg(feature = "oct")]
#[cfg_attr(doc, doc(cfg(feature = "oct")))]
impl<const N: usize> DecodeBorrowed<str> for String<N> { }

impl<const N: usize> Default for String<N> {
	#[inline(always)]
	fn default() -> Self {
		let buf = [Default::default(); N];
		let len = 0x0;

		unsafe { Self::from_raw_parts(buf, len) }
	}
}

impl<const N: usize> Deref for String<N> {
	type Target = str;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<const N: usize> DerefMut for String<N> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_str()
	}
}

impl<const N: usize> Display for String<N> {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(self.as_str(), f)
	}
}

#[cfg(feature = "oct")]
#[cfg_attr(doc, doc(cfg(feature = "oct")))]
impl<const N: usize> Encode for String<N> {
	type Error = <str as Encode>::Error;

	/// Encodes using the same format as <code>&lt;[prim@str] as Encode&gt;::encode</code>.
	#[inline]
	#[track_caller]
	fn encode(&self, output: &mut encode::Output) -> Result<(), Self::Error> {
		(**self).encode(output)
	}
}

impl<const N: usize> Eq for String<N> { }

impl<const N: usize> FromIterator<char> for String<N> {
	#[inline]
	fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
		let mut buf = [0x00; N];
		let mut len = 0x0;

		for c in iter {
			let rem = N - len;
			let req = c.len_utf8();

			if rem < req { break }

			let start = len;
			let end   = start + req;

			c.encode_utf8(&mut buf[start..end]);

			len += req;
		}

		// SAFETY: All octets are initialised and come from
		// `char::encode_utf8`.
		unsafe { Self::from_raw_parts(buf, len) }
	}
}

impl<const N: usize> FromStr for String<N> {
	type Err = LengthError;

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s)
	}
}

impl<const N: usize> Hash for String<N> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl<I: SliceIndex<str>, const N: usize> Index<I> for String<N> {
	type Output	= I::Output;

	#[inline(always)]
	#[track_caller]
	fn index(&self, index: I) -> &Self::Output {
		self.get(index).unwrap()
	}
}

impl<I: SliceIndex<str>, const N: usize> IndexMut<I> for String<N> {
	#[inline(always)]
	#[track_caller]
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		self.get_mut(index).unwrap()
	}
}

impl<const N: usize> Ord for String<N> {
	#[inline(always)]
	fn cmp(&self, other: &Self) -> Ordering {
		(**self).cmp(&**other)
	}
}

impl<const N: usize, const M: usize> PartialEq<String<M>> for String<N> {
	#[inline(always)]
	fn eq(&self, other: &String<M>) -> bool {
		**self == **other
	}
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl<const N: usize> PartialEq<Cow<'_, str>> for String<N> {
	#[inline(always)]
	fn eq(&self, other: &Cow<str>) -> bool {
		**self == **other
	}
}

impl<const N: usize> PartialEq<str> for String<N> {
	#[inline(always)]
	fn eq(&self, other: &str) -> bool {
		**self == *other
	}
}

impl<const N: usize> PartialEq<&str> for String<N> {
	#[inline(always)]
	fn eq(&self, other: &&str) -> bool {
		**self == **other
	}
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl<const N: usize> PartialEq<alloc::string::String> for String<N> {
	#[inline(always)]
	fn eq(&self, other: &alloc::string::String) -> bool {
		*self == **other
	}
}

impl<const N: usize, const M: usize> PartialOrd<String<M>> for String<N> {
	#[inline(always)]
	fn partial_cmp(&self, other: &String<M>) -> Option<Ordering> {
		(**self).partial_cmp(other)
	}

	#[inline(always)]
	fn lt(&self, other: &String<M>) -> bool {
		**self < **other
	}

	#[inline(always)]
	fn le(&self, other: &String<M>) -> bool {
		**self <= **other
	}

	#[inline(always)]
	fn gt(&self, other: &String<M>) -> bool {
		**self > **other
	}

	#[inline(always)]
	fn ge(&self, other: &String<M>) -> bool {
		**self >= **other
	}
}

#[cfg(feature = "oct")]
#[cfg_attr(doc, doc(cfg(feature = "oct")))]
impl<const N: usize> SizedEncode for String<N> {
	const MAX_ENCODED_SIZE: usize =
		usize::MAX_ENCODED_SIZE
		+ u8::MAX_ENCODED_SIZE * N;
}

#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
impl<const N: usize> ToSocketAddrs for String<N> {
	type Iter = <str as ToSocketAddrs>::Iter;

	#[inline(always)]
	fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
		self.as_str().to_socket_addrs()
	}
}

impl<const N: usize> TryFrom<char> for String<N> {
	type Error = <Self as FromStr>::Err;

	#[inline(always)]
	fn try_from(value: char) -> Result<Self, Self::Error> {
		let mut buf = [0x00; 0x4];
		let s = value.encode_utf8(&mut buf);

		s.parse()
	}
}

impl<const N: usize> TryFrom<&str> for String<N> {
	type Error = <Self as FromStr>::Err;

	#[inline(always)]
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl<const N: usize> TryFrom<alloc::string::String> for String<N> {
	type Error = <Self as FromStr>::Err;

	#[inline(always)]
	fn try_from(value: alloc::string::String) -> Result<Self, Self::Error> {
		Self::new(&value)
	}
}

/// See [`into_boxed_str`](String::into_boxed_str).
#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl<const N: usize> From<String<N>> for Box<str> {
	#[inline(always)]
	fn from(value: String<N>) -> Self {
		value.into_boxed_str()
	}
}

/// See [`into_std_string`](String::into_std_string).
#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl<const N: usize> From<String<N>> for alloc::string::String {
	#[inline(always)]
	fn from(value: String<N>) -> Self {
		value.into_std_string()
	}
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl<const N: usize> PartialEq<String<N>> for alloc::string::String {
	#[inline(always)]
	fn eq(&self, other: &String<N>) -> bool {
		self.as_str() == other.as_str()
	}
}

// NOTE: This function is used by the `str` macro
// to circumvent itself using code which may be
// forbidden by the macro user's lints. While this
// function is sound, please do not call it direct-
// ly. It is not a breaking change if it is re-
// moved.
#[doc(hidden)]
#[inline(always)]
#[must_use]
#[track_caller]
pub const fn __string<const N: usize>(s: &'static str) -> String<N> {
	assert!(s.len() <= N, "cannot construct string from literal that is longer");

	// SAFETY: `s` has been tested to not contain more
	// than `N` octets.
	unsafe { String::new_unchecked(s) }
}
