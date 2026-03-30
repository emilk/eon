use core::fmt;

use crate::{Span, TokenKind};

/// Result alias used by formatter-core APIs.
pub type Result<T = ()> = core::result::Result<T, Error>;

/// A formatter-core error with a source span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    /// The byte span where lexing or parsing failed.
    pub span: Span,
    /// The reason lexing or parsing failed.
    pub kind: ErrorKind,
}

impl Error {
    /// Create a new formatter-core error.
    #[inline]
    pub const fn new(span: Span, kind: ErrorKind) -> Self {
        Self { span, kind }
    }

    /// Create a new parse error.
    #[inline]
    pub const fn parse(span: Span, kind: ParseErrorKind) -> Self {
        Self::new(span, ErrorKind::Parse(kind))
    }
}

/// The formatter-core error kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Found an unexpected byte token.
    UnexpectedByte(u8),
    /// Strings must terminate before the end of the document.
    UnterminatedString,
    /// Hidden Unicode that can disguise malicious content is not allowed.
    DisallowedInvisibleUnicode(char),
    /// Parser or formatter-tree construction failure.
    Parse(ParseErrorKind),
}

/// Syntax errors produced while building the formatter-oriented tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// Expected a value token.
    ExpectedValue,
    /// Expected a specific syntax token.
    ExpectedToken(TokenKind),
    /// Found a token that is not valid at this position.
    UnexpectedToken(TokenKind),
    /// Closing token did not match the current container.
    Unbalanced(TokenKind),
    /// Extra tokens remained after parsing the root document.
    TrailingTokens,
    /// Nesting depth exceeded the supported parser bound.
    MaxDepthExceeded,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedByte(byte) => write!(f, "unexpected {}", display_byte(*byte)),
            Self::UnterminatedString => f.write_str("unterminated string"),
            Self::DisallowedInvisibleUnicode(chr) => write!(
                f,
                "Invisible Unicode character U+{:04X} is not allowed in Eon source because it can hide malicious content",
                *chr as u32
            ),
            Self::Parse(kind) => kind.fmt(f),
        }
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedValue => f.write_str("expected a value"),
            Self::ExpectedToken(kind) => write!(f, "expected {kind}"),
            Self::UnexpectedToken(kind) => write!(f, "unexpected {kind}"),
            Self::Unbalanced(kind) => write!(f, "unbalanced {kind}"),
            Self::TrailingTokens => f.write_str("expected end of document"),
            Self::MaxDepthExceeded => f.write_str("maximum recursion depth exceeded"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at bytes {}..{}",
            self.kind, self.span.start, self.span.end
        )
    }
}

fn display_byte(byte: u8) -> ByteDisplay {
    ByteDisplay(byte)
}

struct ByteDisplay(u8);

impl fmt::Display for ByteDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let byte = self.0;
        if byte.is_ascii_graphic() || byte == b' ' {
            write!(f, "`{}`", byte as char)
        } else {
            write!(f, "byte 0x{byte:02X}")
        }
    }
}
