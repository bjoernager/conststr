// Copyright 2025 Gabriel Bjørnager Jensen.

mod test;

#[must_use]
#[track_caller]
pub(crate) const fn decode_utf8(buf: &str, index: usize) -> (char, usize) {
	// Test that the index is valid.

	assert!(buf.is_char_boundary(index));

	let (_, buf) = buf.as_bytes().split_at(index);

	let prefix = buf[0x0];
	let len    = utf8_char_len(prefix);

	let c = match (len, buf) {
		(0x1, &[o0, ..]) => {
			o0 as u32
		}

		(0x2, &[o0, o1, ..]) => {
			let mut c = 0x0;

			c |= (o0 as u32 ^ 0xC0) << 0x6;
			c |=  o1 as u32 ^ 0x80;

			c
		}

		(0x3, &[o0, o1, o2, ..]) => {
			let mut c = 0x0;

			c |= (o0 as u32 ^ 0xE0) << 0xC;
			c |= (o1 as u32 ^ 0x80) << 0x6;
			c |=  o2 as u32 ^ 0x80;

			c
		}

		(0x4, &[o0, o1, o2, o3, ..]) => {
			let mut c = 0x0;

			c |= (o0 as  u32 ^ 0xF0) << 0x12;
			c |= (o1 as  u32 ^ 0x80) << 0xC;
			c |= (o2 as  u32 ^ 0x80) << 0x6;
			c |=  o3 as  u32 ^ 0x80;

			c
		}

		// NOTE: We may assume that UTF-8 sequences are
		// terminated properly. Currently, we may also
		// assume that 6-octet sequences do not occur.
		_ => unreachable!(),
	};

	debug_assert!(char::from_u32(c).is_some());

	// SAFETY: We may assume that the input string
	// only contains UTF-8. In that case our transfor-
	// mation should be correct.
	let c = unsafe { char::from_u32_unchecked(c) };

	(c, len)
}

#[inline]
#[must_use]
#[track_caller]
pub(crate) const fn utf8_char_len(mut prefix: u8) -> usize {
	// By definiton, the two greatest bits of any UTF-8
	// octet are never a part of the prefix.

	match prefix {
		0b00000000..=0b01111111 => {
			0x1
		}

		_ => {
			const MASK: u8 = 0b11111100;

			prefix &= MASK;

			debug_assert!(
				prefix != 0b10000000,
				"cannot get character length from non-prefix",
			);

			prefix.leading_ones() as usize
		}
	}
}
