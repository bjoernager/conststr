# Changelog

This is the changelog of [conststr](https://crates.io/crates/conststr/).
See `README.md` for more information.

## 0.1.0

* Add `oct` feature
* Implement `Encode`, `Decode`, `SizedEncode`, and `DecodeBorrowed<str>` for `String`
* Update tests

## 0.0.0

* Add Cargo manifest
* Add changelog
* Add `String` type
* Add `alloc` and `std` features
* Add `string` macro
* Add `error` module
* Dual-license under MIT or Apache 2.0
* Configure lints
* Add gitignore
* Add `LengthError` and `Utf8Error` errors
* Add tests
* Add `new` and `from_utf8` constructors to `String`
* Add `new_unchecked`, `from_utf8_unchecked`, and `from_raw_parts` constructors to `String`
* Add `len` and `is_empty` methods to `String`
* Add `as_ptr`, `as_mut_ptr`, `as_bytes`, `as_bytes_mut`, `as_str` and `as_mut_str` methods to `String`
* Add `into_std_string`, `into_boxed_str`, and `into_raw_parts` destructors to `String`
* Implement `AsRef<str>`, `AsMut<str>`, `AsRef<[u8]>`, `AsRef<OsStr>`, and `AsRef<Path>` for `String`
* Implement `Borrow<str>` and `BorrowMut<str>` for `String`
* Implement `Deref` and `DerefMut` for `String`
* Implement `PartialOrd`, `Eq`, and `Ord` for `String`
* Implement `FromStr` for `String`
* Implement `ToSocketAddrs` for `String`
* Implement `TryFrom<char>`, `TryFrom<&str>`, and `TryFrom<alloc::string::String>` for `String`
* Implement `PartialEq<Self>`, `PartialEq<str>`, `PartialEq<&str>`, `PartialEq<Cow>`, and `PartialEq<alloc::string::String>` for `String`
* Implement `Default` for `String`
* Implement `Debug` and `Display` for `String`
* Implement `FromIterator<char>` for `String`
* Implement `Hash` for `String`
* Implement `Index` and `IndexMut` for `String`
* Implement `From<String>` for `alloc::string::String` and `Box<str>`
* Implement `PartialEq<String>` for `alloc::string::String`
* Add `is_char_boundary` method to `String`
* Implement `Clone` and `Copy` for `String`
