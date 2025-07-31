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
//! Deserialize any value that implements `serde::Deserialize` using
//! [`eon::from_str`].
//!
//! Serialize any value that implements `serde::Serialize` into Eon using [`Self::to_string`]
//!
//! ## Usage with [`Value`]
//! You can also treat an Eon document as a dynamically types [`Value`].
//!
//! Load an Eon document into a [`Value`] using [`Value::from_str`].
//! Serialize a [`Value`] into an Eon string using [`Value::format`].
//!
//! ## Reading/writing comments
//! An Eon document can contain comments, which are NOT part of the [`Value`] type.
//! To load and serialize comments, use the low-level [`eon_syntax`] crate instead.
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

/// Exported external crates used by [`eon`].
pub mod external {
    pub use eon_syntax;
    pub use vec1;
}

#[cfg(feature = "serde")]
pub use self::serde::{SerializationError, from_str, to_string, to_value};
