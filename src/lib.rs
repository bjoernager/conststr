// Copyright 2025 Gabriel Bjørnager Jensen.

//! Constant strings.

#![no_std]

#![warn(missing_docs)]

#![cfg_attr(doc, feature(doc_cfg))]

extern crate self as conststr;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod error;

mod string;

pub use string::{__string, String};

/// Directly constructs a [`String`](crate::string::String) object.
///
/// This macro tests at compile-time whether the string literal can fit into the inferred length.
/// Compilation will fail if this is not the case.
#[macro_export]
macro_rules! string {
	($s:expr) => {
		const { conststr::__string($s) }
	};

	() => {
		const { conststr::__string("") }
	};
}
