//! A from-scratch JSON library for Rust.
//!
//! `no_std` + `alloc` by default; enable the `std` feature (on by default)
//! for `Display`/`Error` impls and other std-only ergonomics.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod error;
mod number;
mod parser;
mod ser;
mod value;

pub use error::{Category, Error};
pub use number::Number;
pub use value::{Map, Value};

use parser::Parser;

/// Shorthand for `Result<T, Error>`, matching this crate's error type.
pub type Result<T> = core::result::Result<T, Error>;

/// Parses a JSON value from a string slice.
pub fn from_str(s: &str) -> Result<Value> {
    Parser::parse(s)
}

pub use ser::{to_string, to_string_pretty};
