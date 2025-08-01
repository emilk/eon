//! # Eon: the human-friendly configuration format
//! Eon is a human-friendly configuration format that is designed to be easy to read and write.
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
//! Read more at <https://github.com/emilk/eon>.
//!
//! This crate provides a parser and formatter for Eon.
//! It is used to implement the [`eonfmt`](http://crates.io/crates/eonfmt) CLI tool,
//! but also used by the [`eon`](http://crates.io/crates/eon) crate to parse and format Eon documents.
//!
//! You can use it to read and write Eon documents, with comments.
//! This can be useful for e.g. reading "docstrings" from an `.eon` file,
//! or for automate the editing an `.eon` file while preserving comments and formatting.

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
    strings::{escape_and_quote, is_valid_identifier, unescape_and_unquote},
    token_tree::{TokenKeyValue, TokenList, TokenMap, TokenTree, TokenValue, TokenVariant},
};

/// Parses an Eon file and re-indents and formats it in a pretty way.
///
/// ## Errors
/// Returns an error if the source is not valid Eon syntax.
pub fn reformat(eon_source: &str, options: &FormatOptions) -> Result<String> {
    TokenTree::parse_str(eon_source).map(|value| value.format(options))
}
