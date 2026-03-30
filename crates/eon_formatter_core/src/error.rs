use core::fmt;

use crate::Span;

/// Result alias used by the formatter-oriented lexer.
pub type Result<T = ()> = core::result::Result<T, Error>;

/// A lexer error with a source span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    /// The byte span where lexing failed.
    pub span: Span,
    /// The reason lexing failed.
    pub kind: ErrorKind,
}

impl Error {
    /// Create a new lexer error.
    #[inline]
    pub const fn new(span: Span, kind: ErrorKind) -> Self {
        Self { span, kind }
    }
}

/// The formatter-core lexer error kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Found an unexpected byte token.
    UnexpectedByte(u8),
    /// Strings must terminate before the end of the document.
    UnterminatedString,
    /// Hidden Unicode that can disguise malicious content is not allowed.
    DisallowedInvisibleUnicode(char),
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
