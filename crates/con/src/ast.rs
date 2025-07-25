//! Implemented an Abstract Syntax Tree (AST), inclduing comments in the source code.
//!
//! The comments are preserved for the benefit of the formatter.

use std::borrow::Cow;

use crate::span::Span;

/// `// A comment`.
///
/// The string includes the slashes, but not the trailing newline (if any).
pub type Comment<'s> = &'s str;

#[derive(Debug)]
pub struct CommentedValue<'s> {
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
    pub value: AstValue<'s>,

    /// Comment after the value on the same line.
    ///
    /// `value // Like this`.
    pub suffix_comment: Option<Comment<'s>>,
}

#[derive(Debug)]
pub struct CommentedKeyValue<'s> {
    /// Comments on proceeding lines.
    ///
    /// ```ignore
    /// // Like this
    /// // and this.
    /// key: calue
    /// ```
    pub prefix_comments: Vec<Comment<'s>>,

    /// The key of the key-value pair.
    pub key: AstValue<'s>,

    /// The value of the key-value pair.
    pub value: AstValue<'s>,

    /// Comment after the value on the same line.
    ///
    /// `key: value // Like this`.
    pub suffix_comment: Option<Comment<'s>>,
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
    pub values: Vec<CommentedValue<'s>>,

    /// Any comments after the last value, before the closing `]`.
    pub closing_comments: Vec<Comment<'s>>,
}

#[derive(Debug)]
pub enum AstValue<'s> {
    /// `null`, `true`, or `false`, or the key of an object
    Identifier(Cow<'s, str>),

    /// Anything that starts with a sign (+/-) or a digit (0-9).
    Number(Cow<'s, str>),

    /// Includes the actual quotes of the string, both opening and closing.
    ///
    /// Any special characters are escaped, e.g. `\n`, `\t`, `\"`, etc.
    QuotedString(Cow<'s, str>),

    /// A list, like `[ a, b, c, … ]`.
    List(CommentedList<'s>),

    /// An object, like `{ key: value }`.
    Object(CommentedMap<'s>),
}
