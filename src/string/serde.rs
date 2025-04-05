// Copyright 2025 Gabriel Bjørnager Jensen.

#![cfg(feature = "serde")]

use crate::String;

use core::fmt::{self, Formatter};
use core::marker::PhantomData;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Default)]
struct StringVisitor<const N: usize> {
	_s: PhantomData<fn() -> String<N>>,
}

impl<const N: usize> StringVisitor<N> {
	#[inline(always)]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			_s: PhantomData,
		}
	}
}

impl<'de, const N: usize> Visitor<'de> for StringVisitor<N> {
	type Value = String<N>;

	#[inline]
	fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
		write!(formatter, "a string of at most `{N}` octets")
	}

	#[inline]
	fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
		String::new(v).map_err(E::custom)
	}
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<'de, const N: usize> Deserialize<'de> for String<N> {
	#[inline]
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_str(StringVisitor::new())
	}
}

#[cfg_attr(doc, doc(cfg(feature = "serde")))]
impl<const N: usize> Serialize for String<N> {
	#[inline]
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self)
	}
}
