//! This module describes the structure of a Con document,
//! including comments.
//!
//! The comments are preserved for the benefit of the formatter.

use std::borrow::Cow;

use crate::span::Span;

/// `// A comment`.
///
/// The string includes the slashes, but not the trailing newline (if any).
pub type Comment<'s> = &'s str;

/// A tree of tokens, representing the structure of the Con source code, including comments.
#[derive(Debug)]
pub struct TokenTree<'s> {
    /// The span of the token tree in the source code.
    pub span: Span,

    /// Comments on proceeding lines.
    ///
    /// ```ignore
    /// // Like this
    /// // and this.
    /// 42
    /// ```
    pub prefix_comments: Vec<Comment<'s>>,

    /// The actual value.
    pub value: TreeValue<'s>,

    /// Comment after the value on the same line.
    ///
    /// `value // Like this`.
    pub suffix_comment: Option<Comment<'s>>,
}

#[derive(Debug)]
pub struct CommentedKeyValue<'s> {
    /// The key of the key-value pair.
    pub key: TokenTree<'s>,

    /// The value of the key-value pair.
    pub value: TokenTree<'s>,
}

/// An object, like `{ key: value, … }`.
#[derive(Debug)]
pub struct CommentedMap<'s> {
    pub key_values: Vec<CommentedKeyValue<'s>>,

    /// Any comments after the last `key: value` pair, before the closing `}`.
    pub closing_comments: Vec<Comment<'s>>,
}

/// A list, like `[ a, b, c, … ]`.
#[derive(Debug)]
pub struct CommentedList<'s> {
    pub values: Vec<TokenTree<'s>>,

    /// Any comments after the last value, before the closing `]`.
    pub closing_comments: Vec<Comment<'s>>,
}

/// A sum-type choice, like `Rgb(255, 0, 0)`.
#[derive(Debug)]
pub struct CommentedChoice<'s> {
    /// Span of just the name
    pub name_span: Span,

    /// The name of the choice, like `Rgb`.
    pub name: Cow<'s, str>,

    /// The contents of the choice, like `255, 0, 0`.
    pub values: Vec<TokenTree<'s>>,

    /// Any comments after the last value, before the closing `]`.
    pub closing_comments: Vec<Comment<'s>>,
}

#[derive(Debug)]
pub enum TreeValue<'s> {
    /// `null`, `true`, or `false`, or the key of an map
    Identifier(Cow<'s, str>),

    /// Anything that starts with a sign (+/-) or a digit (0-9).
    Number(Cow<'s, str>),

    /// Includes the actual quotes of the string, both opening and closing.
    ///
    /// Any special characters are escaped, e.g. `\n`, `\t`, `\"`, etc.
    QuotedString(Cow<'s, str>),

    /// A list, like `[ a, b, c, … ]`.
    List(CommentedList<'s>),

    /// An map, like `{ key: value }`.
    Map(CommentedMap<'s>),

    /// A sum-type choice, like `Rgb(…)`
    Choice(CommentedChoice<'s>),
}
