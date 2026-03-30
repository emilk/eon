use alloc::vec::Vec;

use crate::Span;

/// Borrowed formatter input preserving the exact token/trivia order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenStream<'a> {
    /// Interleaved trivia and tokens as they appeared in the source.
    pub items: Vec<Item<'a>>,
}

/// A single borrowed formatter item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item<'a> {
    /// A meaningful syntax token.
    Token(Token<'a>),
    /// Whitespace or comments between tokens.
    Trivia(Trivia<'a>),
}

/// A single borrowed syntax token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    /// Token byte span.
    pub span: Span,
    /// Token kind.
    pub kind: TokenKind,
    /// Raw token text borrowed from the source.
    pub raw: &'a str,
}

/// The token kinds preserved for formatting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// `[`
    OpenList,
    /// `]`
    CloseList,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `[a-zA-Z_][a-zA-Z0-9_]*`
    Identifier,
    /// `[+\-0-9\.][0-9a-zA-Z\.+\-_]*`
    Number,
    /// Any of the four quoted string token families.
    String(StringKind),
}

/// The supported quoted string token families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringKind {
    /// `"basic"`
    Basic,
    /// `'literal'`
    Literal,
    /// `"""multiline basic"""`
    MultilineBasic,
    /// `'''multiline literal'''`
    MultilineLiteral,
}

/// Trivia preserved between tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trivia<'a> {
    /// Trivia byte span.
    pub span: Span,
    /// Trivia kind.
    pub kind: TriviaKind,
    /// Raw trivia text borrowed from the source.
    pub raw: &'a str,
    /// Number of logical line breaks in this trivia.
    pub line_breaks: usize,
}

/// The trivia kinds preserved for formatting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    /// Spaces, tabs, form-feeds, and line breaks.
    Whitespace,
    /// `// comment`
    Comment,
}
