//! # Con: the human-friendly configuration format
//! TODO: fill this in
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]

mod ast_from_value;
mod value;
mod value_from_ast;

#[cfg(feature = "serde")]
mod serde;

pub use {
    crate::value::{Map, Number, Value},
    con_syntax::{Error, FormatOptions, Result, reformat},
};

#[cfg(feature = "serde")]
pub use self::serde::{SerializationError, from_str, to_string, to_value};
