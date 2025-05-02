// Copyright 2025 Gabriel Bjørnager Jensen.

#![cfg(test)]

use core::cmp::Ordering;
use conststr::{String, string};
use conststr::error::{LengthError, Utf8Error};
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

#[test]
fn test_string_insert() {
	let mut s = String::<0xC>::new();

	assert_eq!(s.insert(0x0, '\u{1F480}'), Ok(()));
	assert_eq!(s.len(),                    0x4);
	assert_eq!(s,                          "\u{1F480}");

	assert_eq!(s.insert(0x0, '\u{130BA}'), Ok(()));
	assert_eq!(s.len(),                    0x8);
	assert_eq!(s,                          "\u{130BA}\u{1F480}");

	assert_eq!(s.insert(0x4,  '\u{81A3}'), Ok(()));
	assert_eq!(s.len(),                    0xB);
	assert_eq!(s,                          "\u{130BA}\u{81A3}\u{1F480}");

	assert_eq!(s.insert(0xB,  '!'),        Ok(()));
	assert_eq!(s.len(),                    0xC);
	assert_eq!(s,                          "\u{130BA}\u{81A3}\u{1F480}!");

	assert_eq!(s.push('?'),                Err(LengthError { remaining: 0x0, count: 0x1 }));
	assert_eq!(s.len(),                    0xC);
	assert_eq!(s,                          "\u{130BA}\u{81A3}\u{1F480}!");
}

#[test]
fn test_string_push_pop() {
	let mut s = String::<0x8>::new();

	assert_eq!(s.push('\u{FFFD}'), Ok(()));
	assert_eq!(s.len(),            0x3);

	assert_eq!(s.push('\u{0D9E}'), Ok(()));
	assert_eq!(s.len(),            0x6);

	assert_eq!(s.push('\u{0394}'), Ok(()));
	assert_eq!(s.len(),            0x8);

	assert_eq!(s.push('!'),        Err(LengthError { remaining: 0x0, count: 0x1 }));
	assert_eq!(s.len(),            0x8);

	assert_eq!(s.pop(), Some('\u{0394}'));
	assert_eq!(s.pop(), Some('\u{0D9E}'));
	assert_eq!(s.pop(), Some('\u{FFFD}'));
	assert_eq!(s.pop(), None);
}

#[test]
fn test_string_remove() {
	let mut s: String<0x8> = string!("Ma\u{00F1}o\u{351E}");

	assert_eq!(s.len(),       0x8);
	assert_eq!(s,             "Ma\u{00F1}o\u{351E}");

	assert_eq!(s.remove(0x4), 'o');
	assert_eq!(s.len(),       0x7);
	assert_eq!(s,             "Ma\u{00F1}\u{351E}");

	assert_eq!(s.remove(0x2), '\u{00F1}');
	assert_eq!(s.len(),       0x5);
	assert_eq!(s,             "Ma\u{351E}");

	assert_eq!(s.remove(0x0), 'M');
	assert_eq!(s.len(),       0x4);
	assert_eq!(s,             "a\u{351E}");

	assert_eq!(s.remove(0x1), '\u{351E}');
	assert_eq!(s.len(),       0x1);
	assert_eq!(s,             "a");

	assert_eq!(s.remove(0x0), 'a');
	assert_eq!(s.len(),       0x0);
	assert_eq!(s,             "");
}

#[test]
#[should_panic]
fn test_string_remove_non_boundary() {
	let mut s: String<0x4> = string!("\u{5149}");

	let _ = s.remove(0x2);
}
