//! This module describes the structure of a Eon document,
//! including comments.
//!
//! The comments are preserved for the benefit of the formatter.

use std::borrow::Cow;

use crate::span::Span;

/// `// A comment`.
///
/// The string includes the slashes, but not the trailing newline (if any).
pub type Comment<'s> = &'s str;

/// A tree of tokens, representing the structure of the Eon source code, including comments.
///
/// This is actually something between a Concrete Syntax Tree (CST) and an Abstract Syntax Tree (AST),
/// in that it preserves comments, but discards whitespace and some optional tokens (like commas after list items),
/// though those can be inferred from the [`Span`] of the tokens, together with the original
/// Eon file
#[derive(Debug)]
pub struct TokenTree<'s> {
    /// The span of this token tree in the source code, if known.
    pub span: Option<Span>,

    /// Comments on proceeding lines.
    ///
    /// ```ignore
    /// // Like this
    /// // and this.
    /// 42
    /// ```
    pub prefix_comments: Vec<Comment<'s>>,

    /// The actual value.
    pub value: TokenValue<'s>,

    /// Comment after the value on the same line.
    ///
    /// `value // Like this`.
    pub suffix_comment: Option<Comment<'s>>,
}

#[derive(Debug)]
pub struct TokenKeyValue<'s> {
    /// The key of the key-value pair.
    pub key: TokenTree<'s>,

    /// The value of the key-value pair.
    pub value: TokenTree<'s>,
}

/// An object, like `{ key: value, … }`.
#[derive(Debug)]
pub struct TokenMap<'s> {
    pub key_values: Vec<TokenKeyValue<'s>>,

    /// Any comments after the last `key: value` pair, before the closing `}`.
    pub closing_comments: Vec<Comment<'s>>,
}

/// A list, like `[ a, b, c, … ]`.
#[derive(Debug)]
pub struct TokenList<'s> {
    pub values: Vec<TokenTree<'s>>,

    /// Any comments after the last value, before the closing `]`.
    pub closing_comments: Vec<Comment<'s>>,
}

/// A sum-type (enum) variant
#[derive(Debug)]
pub struct TokenVariant<'s> {
    /// Span of just the name
    pub name_span: Option<Span>,

    /// The quoted name of the variant, like `"Rgb"`.
    pub quoted_name: Cow<'s, str>,

    /// The contents of the variant, like `255, 0, 0`.
    pub values: Vec<TokenTree<'s>>,

    /// Any comments after the last value, before the closing `]`.
    pub closing_comments: Vec<Comment<'s>>,
}

#[derive(Debug)]
pub enum TokenValue<'s> {
    /// `null`, `true`, or `false`, or the key of an map (when quotes aren't needed).
    Identifier(Cow<'s, str>),

    /// Anything that starts with a sign (+/-) or a digit (0-9).
    Number(Cow<'s, str>),

    /// Includes the actual quotes of the string, both opening and closing.
    ///
    /// Can be on of:
    /// - `"Basic string"`
    /// - `'Literal string'`
    /// - `"""Multiline basic string"""`
    /// - `'''Multiline literal string'''`
    QuotedString(Cow<'s, str>),

    /// A list, like `[ a, b, c, … ]`.
    List(TokenList<'s>),

    /// An map, like `{ key: value }`.
    Map(TokenMap<'s>),

    /// A sum-type (enum) variant
    Variant(TokenVariant<'s>),
}

impl TokenValue<'_> {
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
}

impl<'s> From<TokenValue<'s>> for TokenTree<'s> {
    fn from(value: TokenValue<'s>) -> Self {
        TokenTree {
            span: None,
            prefix_comments: vec![],
            value,
            suffix_comment: None,
        }
    }
}
