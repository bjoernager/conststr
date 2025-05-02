// Copyright 2025 Gabriel Bjørnager Jensen.

#![cfg(test)]

use crate::utf8::{decode_utf8, utf8_char_len};

#[test]
fn test_decode_utf8() {
	assert_eq!(
		decode_utf8("\0", 0x0),
		('\0', 0x1),
	);

	assert_eq!(
		decode_utf8("\u{00B1}", 0x0),
		('\u{00B1}', 0x2),
	);

	assert_eq!(
		decode_utf8("\u{FDF2}", 0x0),
		('\u{FDF2}', 0x3),
	);

	assert_eq!(
		decode_utf8("\u{1F54B}", 0x0),
		('\u{1F54B}', 0x4),
	);
}

#[test]
fn test_utf8_char_len() {
	assert_eq!(utf8_char_len(0b01111111u8), 0x1);
	assert_eq!(utf8_char_len(0b11011111u8), 0x2);
	assert_eq!(utf8_char_len(0b11101111u8), 0x3);
	assert_eq!(utf8_char_len(0b11110111u8), 0x4);
	assert_eq!(utf8_char_len(0b11111011u8), 0x5);
	assert_eq!(utf8_char_len(0b11111111u8), 0x6);
}
