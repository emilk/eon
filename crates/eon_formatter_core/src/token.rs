use alloc::vec::Vec;
use core::fmt;

use crate::Span;

/// Borrowed formatter input preserving the exact token/trivia order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenStream<'a> {
    /// The full borrowed source text.
    pub source: &'a str,
    /// Interleaved trivia and tokens as they appeared in the source.
    pub items: Vec<Item<'a>>,
    pub(crate) token_item_indices: Vec<usize>,
}

impl<'a> TokenStream<'a> {
    /// Create a token stream from already lexed items.
    pub fn new(source: &'a str, items: Vec<Item<'a>>) -> Self {
        let token_item_indices = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                Item::Token(_) => Some(index),
                Item::Trivia(_) => None,
            })
            .collect();

        Self {
            source,
            items,
            token_item_indices,
        }
    }

    /// The number of syntax tokens in the stream.
    pub fn token_count(&self) -> usize {
        self.token_item_indices.len()
    }

    /// Return one syntax token by zero-based token index.
    pub fn token(&self, token_index: usize) -> Option<Token<'a>> {
        let item_index = *self.token_item_indices.get(token_index)?;
        let Item::Token(token) = self.items[item_index] else {
            unreachable!("token_item_indices always point at tokens");
        };
        Some(token)
    }

    /// Returns `true` if the stream contains no syntax tokens.
    pub fn has_no_tokens(&self) -> bool {
        self.token_item_indices.is_empty()
    }

    /// Iterate over syntax tokens with formatter-oriented trivia accessors.
    pub fn tokens(&self) -> crate::TokenRefs<'_, 'a> {
        crate::TokenRefs::new(self)
    }

    /// Borrow one token view by zero-based token index.
    pub fn token_ref(&self, token_index: usize) -> Option<crate::TokenRef<'_, 'a>> {
        if token_index < self.token_count() {
            Some(crate::TokenRef::new(self, token_index))
        } else {
            None
        }
    }
}

impl Default for TokenStream<'_> {
    fn default() -> Self {
        Self::new("", Vec::new())
    }
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

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenList => f.write_str("'['"),
            Self::CloseList => f.write_str("']'"),
            Self::OpenBrace => f.write_str("'{'"),
            Self::CloseBrace => f.write_str("'}'"),
            Self::OpenParen => f.write_str("'('"),
            Self::CloseParen => f.write_str("')'"),
            Self::Colon => f.write_str("':'"),
            Self::Comma => f.write_str("','"),
            Self::Identifier => f.write_str("identifier"),
            Self::Number => f.write_str("number"),
            Self::String(_) => f.write_str("string"),
        }
    }
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
