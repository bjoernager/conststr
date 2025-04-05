// Copyright 2025 Gabriel Bjørnager Jensen.

#![cfg(test)]

use core::cmp::Ordering;
use conststr::{String, string};
use conststr::error::Utf8Error;
use oct::decode::{Decode, Input};

#[test]
fn test_string() {
	let s: String::<0x4> = "hello world".chars().collect();
	assert_eq!(s, "hell")
}

#[test]
fn test_string_decode() {
	let data = *b"\x0C\x00if constexpr";

	let mut input = Input::new(&data);

	let s = String::<0x100>::decode(&mut input).unwrap();

	assert_eq!(
		s,
		"if constexpr",
	);
}

#[test]
fn test_string_macro() {
	let s0: String<0x08> = string!("conststr");
	let s1: String<0x10> = string!("conststr");

	assert_eq!(s0, s1);
}

#[test]
fn test_string_serde() {
	use serde_test::{assert_tokens, Token};

	let s = String::<0x7F>::new("I\u{2764}serde").unwrap();

	assert_tokens(
		&s,
		&[
			Token::Str("I\u{2764}serde"),
		],
	);
}

#[test]
fn test_string_size() {
	let s0: String<0x0C> = string!("Hello there!");
	let s1: String<0x12> = string!("MEIN_GRO\u{1E9E}_GOTT");
	let s2: String<0x05> = string!("Hello");

	assert_eq!(s0.partial_cmp(&s0), Some(Ordering::Equal));
	assert_eq!(s0.partial_cmp(&s1), Some(Ordering::Less));
	assert_eq!(s0.partial_cmp(&s2), Some(Ordering::Greater));

	assert_eq!(s1.partial_cmp(&s0), Some(Ordering::Greater));
	assert_eq!(s1.partial_cmp(&s1), Some(Ordering::Equal));
	assert_eq!(s1.partial_cmp(&s2), Some(Ordering::Greater));

	assert_eq!(s2.partial_cmp(&s0), Some(Ordering::Less));
	assert_eq!(s2.partial_cmp(&s1), Some(Ordering::Less));
	assert_eq!(s2.partial_cmp(&s2), Some(Ordering::Equal));

	assert_eq!(s0, "Hello there!");
	assert_eq!(s1, "MEIN_GRO\u{1E9E}_GOTT");
	assert_eq!(s2, "Hello");
}

#[test]
fn test_string_from_utf8() {
	macro_rules! test_utf8 {
		{
			len: $len:literal,
			utf8: $utf8:expr,
			result: $result:pat$(,)?
		 } => {{
			assert!(matches!(
				const { String::<$len>::from_utf8(*$utf8) },
				$result,
			));
		}};
	}

	test_utf8!(
		len:    0x3,
		utf8:   b"A\xF7c",
		result: Err(Utf8Error { value: 0xF7, index: 0x1 }),
	);

	test_utf8!(
		len:    0x4,
		utf8:   b"A\xC3\xBCc",
		result: Ok(..),
	);

	test_utf8!(
		len:    0x4,
		utf8:   b"20\x20\xAC",
		result: Err(Utf8Error { value: 0xAC, index: 0x3 }),
	);

	test_utf8!(
		len:    0x5,
		utf8:   b"20\xE2\x82\xAC",
		result: Ok(..),
	);
}
