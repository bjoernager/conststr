// Copyright 2025 Gabriel Bjørnager Jensen.

//! Error types.

mod length_error;
mod utf8_error;

pub use length_error::LengthError;
pub use utf8_error::Utf8Error;
