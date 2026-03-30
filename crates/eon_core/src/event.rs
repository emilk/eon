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
