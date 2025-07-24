use crate::{span::Span, token::TokenKind};

/// `// A comment`.
///
/// The string includes the slashes, but not the trailing newline (if any).
pub type Comment<'s> = &'s str;

#[derive(Clone, Copy, Debug)]
pub struct PlacedToken<'s> {
    /// The span of the token in the input string.
    pub span: Span,

    /// The token value
    pub slice: &'s str,

    /// The token type
    pub kind: TokenKind,
}

#[derive(Debug)]
pub struct CommentedValue<'s> {
    pub span: Span,

    /// Comments on preceeding lines.
    ///
    /// ```ignore
    /// // Like this
    /// // and this.
    /// 42
    /// ```
    pub prefix_comments: Vec<Comment<'s>>,

    /// The actual value.
    pub value: Value<'s>,

    /// Comment after the value on the same line.
    ///
    /// `value // Like this`.
    pub suffix_comment: Option<Comment<'s>>,
}

#[derive(Debug)]
pub struct KeyValue<'s> {
    /// The key of the key-value pair.
    ///
    /// Includes quotes if the key is a quoted string.
    pub key: PlacedToken<'s>,

    /// The value of the key-value pair.
    ///
    /// Also contains any comments before the key, before the value, and after the value.
    pub value: CommentedValue<'s>,
}

/// An object, like `{ key: value, … }`.
#[derive(Debug)]
pub struct Object<'s> {
    pub key_values: Vec<KeyValue<'s>>,

    /// Any comments after the last `key: value` pair, before the closing `}`.
    pub closing_comments: Vec<Comment<'s>>,
}

/// A list, like `[ a, b, c, … ]`.
#[derive(Debug)]
pub struct List<'s> {
    pub values: Vec<CommentedValue<'s>>,

    /// Any comments after the last value, before the closing `]`.
    pub closing_comments: Vec<Comment<'s>>,
}

#[derive(Debug)]
pub enum Value<'s> {
    /// `null`, `true`, or `false`.
    Identifier(&'s str),

    /// Includes the actual quotes of the string, both opening and closing.
    String(&'s str),

    /// A list, like `[ a, b, c, … ]`.
    List(List<'s>),

    /// An object, like `{ key: value }`.
    Object(Object<'s>),
}
