//! # Con: the human-friendly configuration format
//! TODO: fill this in
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]

pub mod ast;
pub mod error;
pub mod format;
pub mod parse;
pub mod span;
pub mod token;

#[cfg(feature = "serde")]
mod serde;

pub use crate::format::FormatOptions;

pub use crate::error::{Error, Result};

/// Parses a Con file and re-indents and formats it in a pretty way.
pub fn reformat(input: &str, options: &FormatOptions) -> Result<String> {
    let value = parse::parse_top_str(input)?;
    Ok(format::format(&value, options))
}
