//! # Con: the human-friendly configuration format
//! TODO: fill this in
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]

mod token_tree_from_value;
mod value;
mod value_from_token_tree;

#[cfg(feature = "serde")]
mod serde;

pub use {
    crate::value::{Map, Number, Value, Variant},
    con_syntax::{Error, FormatOptions, Result, reformat},
};

/// Exported external crates used by.
pub mod external {
    pub use con_syntax;
}

#[cfg(feature = "serde")]
pub use self::serde::{SerializationError, from_str, to_string, to_value};
