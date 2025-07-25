//! # Con: the human-friendly configuration format
//! TODO: fill this in
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]

pub mod ast;
mod ast_from_value;
pub mod error;
pub mod format;
pub mod parse;
pub mod span;
pub mod token;
mod value;
mod value_from_ast;

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "serde")]
pub use self::serde::{SerializationError, from_str, to_string, to_value};

use crate::ast::CommentedValue;
pub use crate::{
    error::{Error, Result},
    format::FormatOptions,
    value::{Map, Number, Value},
};

/// Parses a Con file and re-indents and formats it in a pretty way.
///
/// ## Errors
/// Returns an error if the source is not valid Con syntax.
pub fn reformat(source: &str, options: &FormatOptions) -> Result<String> {
    CommentedValue::parse_str(source).map(|value| value.format(options))
}
