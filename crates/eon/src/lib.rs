//! # Eon: the human-friendly configuration format
//! To learn the Eon syntax, see <https://github.com/emilk/eon>.
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
