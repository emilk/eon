//! # Eon: the human-friendly configuration format
//! TODO: fill this in
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]

mod error;
mod format;
mod parse;
mod span;
mod strings;
mod token_kind;
mod token_tree;

pub use crate::{
    error::{Error, Result},
    format::FormatOptions,
    span::Span,
    strings::{escape_and_quote, key_needs_quotes, unescape_and_unquote},
    token_tree::{TokenKeyValue, TokenList, TokenMap, TokenTree, TokenValue, TokenVariant},
};

/// Parses a Eon file and re-indents and formats it in a pretty way.
///
/// ## Errors
/// Returns an error if the source is not valid Eon syntax.
pub fn reformat(eon_source: &str, options: &FormatOptions) -> Result<String> {
    TokenTree::parse_str(eon_source).map(|value| value.format(options))
}
