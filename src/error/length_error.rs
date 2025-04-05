// Copyright 2025 Gabriel Bjørnager Jensen.

use core::convert::Infallible;
use core::error::Error;
use core::fmt::{self, Display, Formatter};

#[cfg(feature = "oct")]
use oct::error::GenericDecodeError;

/// A constant string overflowed its buffer.
#[derive(Debug, Eq, PartialEq)]
#[must_use]
pub struct LengthError {
	/// The remaining capacity of the buffer.
	pub remaining: usize,

	/// The required amount of elements.
	pub count: usize,
}

impl Display for LengthError {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "collection with ({}) remaining size cannot hold ({}) more elements", self.remaining, self.count)
	}
}

impl Error for LengthError { }

impl From<Infallible> for LengthError {
	#[inline(always)]
	fn from(_value: Infallible) -> Self {
		unreachable!()
	}
}

#[cfg(feature = "oct")]
#[cfg_attr(doc, doc(cfg(feature = "oct")))]
impl From<LengthError> for GenericDecodeError {
	#[inline(always)]
	fn from(value: LengthError) -> Self {
		let e = oct::error::LengthError {
			remaining: value.remaining,
			count:     value.count,
		};

		Self::SmallBuffer(e)
	}
}
