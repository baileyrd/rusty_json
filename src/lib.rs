//! A from-scratch JSON library for Rust.
//!
//! `no_std` + `alloc` by default; enable the `std` feature (on by default)
//! for `Display`/`Error` impls and other std-only ergonomics.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod de;
mod error;
mod macros;
mod number;
mod parser;
mod ser;
mod serde_support;
mod value;

pub use error::{Category, Error};
pub use number::Number;
pub use value::{Map, Value};

/// Shorthand for `Result<T, Error>`, matching this crate's error type.
pub type Result<T> = core::result::Result<T, Error>;

pub use de::{from_slice, from_str};
pub use ser::{to_string, to_string_pretty};

/// Not public API. Re-exports used by the [`json!`] macro's expansion so it
/// works from downstream crates without them needing `extern crate alloc`.
#[doc(hidden)]
pub mod __private {
    pub use alloc::vec;
}
