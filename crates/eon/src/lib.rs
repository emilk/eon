//! # Eon: the human-friendly configuration format
//! Eon is a configuration format that is designed to be familiar, clean, and powerful.
//!
//! Example Eon document:
//!
//! ```text
//! // Comment
//! string: "Hello Eon!"
//! list: [1, 2, 3]
//! object: {
//!     boolean: true
//!     regex: '\d{3}-\d{3}-\d{4}'
//! }
//! map: {
//!     1: "map keys don't need to be strings"
//!     2: "they can be any Eon value"
//! }
//! special_floats: [+inf, -inf, +nan]
//! ```
//!
//! Read more about Eon at <https://github.com/emilk/eon>.
//!
//! ## Usage with `serde`
//! Make sure to enable the `serde` feature for `eon` in your `Cargo.toml` (it's currently enabled by default):
//! ```toml
//! [dependencies]
//! eon = { version = "*", features = ["serde"] }
//! ```
//!
//! Deserialize any value that implements `serde::Deserialize` using [`from_str`].
//!
//! Serialize any value that implements `serde::Serialize` into Eon using [`to_string`]
//!
//! ## Usage with [`Value`]
//! You can also treat an Eon document as a dynamically types [`Value`].
//!
//! Load an Eon document into a [`Value`] using [`Value::from_str`](std::str::FromStr::from_str).
//!
//! Serialize a [`Value`] into an Eon string using [`Value::format`].
//!
//! You can also convert anything that implements `serde::Serialize` into a [`Value`] using [`to_value`],
//!
//! ## Reading/writing comments
//! An Eon document can contain comments, which are NOT part of the [`Value`] type.
//! To load and serialize comments, use the low-level [`eon_syntax`] crate instead.
//!
//! ## Formatting Eon files
//! Use [`reformat`] to format an Eon file.
//! You can also use the [`eonfmt`](http://crates.io/crates/eonfmt) CLI tool.
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
#![warn(missing_docs)] // let's keep eon well-documented

mod token_tree_from_value;
mod value;
mod value_from_token_tree;

#[cfg(feature = "serde")]
mod serde;

pub use {
    crate::value::{Map, Number, Value, Variant},
    eon_syntax::{Error, FormatOptions, Result, reformat},
};

/// External crates used by `eon`.
pub mod external {
    pub use eon_syntax;
    pub use vec1;
}

#[cfg(feature = "serde")]
pub use self::serde::{SerializationError, from_str, to_string, to_value};
