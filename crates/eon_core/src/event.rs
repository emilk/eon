use crate::Span;

/// A sink for the borrowed event stream emitted by the parser.
pub trait EventSink<'a> {
    /// The sink-specific error.
    type Error;

    /// Consume a single event.
    fn event(&mut self, event: SpannedEvent<'a>) -> core::result::Result<(), Self::Error>;
}

impl<'a, T> EventSink<'a> for &mut T
where
    T: EventSink<'a> + ?Sized,
{
    type Error = T::Error;

    #[inline]
    fn event(&mut self, event: SpannedEvent<'a>) -> core::result::Result<(), Self::Error> {
        (**self).event(event)
    }
}

/// An event tagged with a source span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpannedEvent<'a> {
    /// Span of the logical token or delimiter that caused the event.
    pub span: Span,
    /// The event payload.
    pub event: Event<'a>,
}

/// The low-level borrowed Eon event stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event<'a> {
    /// Start of a map.
    BeginMap {
        /// True when the map came from the brace-less root form.
        implicit: bool,
    },
    /// End of a map.
    EndMap,
    /// Marker before a map key value.
    MapKey,
    /// Marker before a map value.
    MapValue,
    /// Start of a list.
    BeginList,
    /// End of a list.
    EndList,
    /// Start of a variant payload.
    BeginVariant {
        /// The variant name as it appeared in the source.
        name: VariantName<'a>,
    },
    /// End of a variant payload.
    EndVariant,
    /// A scalar value.
    Scalar(Scalar<'a>),
}

/// The four Eon string token families.
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

/// A borrowed raw string token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringToken<'a> {
    /// The raw slice including delimiters.
    pub raw: &'a str,
    /// Which string token flavor was parsed.
    pub kind: StringKind,
}

impl<'a> StringToken<'a> {
    /// Returns the decoded string contents as a borrowed slice when no
    /// unescaping or line-ending normalization is required.
    ///
    /// This is the zero-copy fast path for higher layers. Strings containing
    /// `\r` return `None` so callers can preserve the current CRLF
    /// normalization behavior by falling back to a slower decoder.
    #[must_use]
    pub fn decoded_if_borrowed(&self) -> Option<&'a str> {
        let inner = match self.kind {
            StringKind::Basic => self.raw.strip_prefix('"')?.strip_suffix('"')?,
            StringKind::Literal => self.raw.strip_prefix('\'')?.strip_suffix('\'')?,
            StringKind::MultilineBasic => {
                self.raw.strip_prefix("\"\"\"")?.strip_suffix("\"\"\"")?
            }
            StringKind::MultilineLiteral => self.raw.strip_prefix("'''")?.strip_suffix("'''")?,
        };

        if inner.contains('\r') {
            return None;
        }

        match self.kind {
            StringKind::Basic | StringKind::MultilineBasic => {
                (!inner.contains('\\')).then_some(inner)
            }
            StringKind::Literal => Some(inner),
            StringKind::MultilineLiteral => Some(inner.strip_prefix('\n').unwrap_or(inner)),
        }
    }

    /// Returns `true` when decoding the token requires escape handling or line
    /// ending normalization.
    #[must_use]
    pub fn requires_decoding(&self) -> bool {
        self.decoded_if_borrowed().is_none()
    }
}

/// A scalar token borrowed from the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scalar<'a> {
    /// `null`
    Null,
    /// `true` or `false`
    Bool(bool),
    /// A raw number token.
    Number(&'a str),
    /// A bare identifier in value position.
    Identifier(&'a str),
    /// A raw quoted string token.
    String(StringToken<'a>),
}

/// The name of a variant payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantName<'a> {
    /// A bare identifier such as `EnumValue`.
    Identifier(&'a str),
    /// A quoted name for non-identifier variant names.
    String(StringToken<'a>),
}

#[cfg(test)]
mod tests {
    use super::{StringKind, StringToken};

    #[test]
    fn decoded_if_borrowed_handles_unescaped_basic_strings() {
        let token = StringToken {
            raw: "\"hello\"",
            kind: StringKind::Basic,
        };

        assert_eq!(token.decoded_if_borrowed(), Some("hello"));
        assert!(!token.requires_decoding());
    }

    #[test]
    fn decoded_if_borrowed_rejects_escaped_basic_strings() {
        let token = StringToken {
            raw: "\"he\\nllo\"",
            kind: StringKind::Basic,
        };

        assert_eq!(token.decoded_if_borrowed(), None);
        assert!(token.requires_decoding());
    }

    #[test]
    fn decoded_if_borrowed_handles_multiline_literal_leading_newline() {
        let token = StringToken {
            raw: "'''\nhello'''",
            kind: StringKind::MultilineLiteral,
        };

        assert_eq!(token.decoded_if_borrowed(), Some("hello"));
        assert!(!token.requires_decoding());
    }

    #[test]
    fn decoded_if_borrowed_rejects_crlf_normalization_cases() {
        let token = StringToken {
            raw: "\"\"\"hello\r\nworld\"\"\"",
            kind: StringKind::MultilineBasic,
        };

        assert_eq!(token.decoded_if_borrowed(), None);
        assert!(token.requires_decoding());
    }
}
