use core::{convert::Infallible, fmt};

use crate::{
    Error, ErrorKind, Event, EventSink, Result, Scalar, Span, SpannedEvent, StringKind,
    StringToken, VariantName,
};

/// Default maximum nesting depth for maps, lists, and variants.
pub const DEFAULT_MAX_DEPTH: usize = 128;

/// A parse error or a sink error emitted while streaming events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError<E> {
    /// Syntax or scanning error.
    Parse(Error),
    /// Error returned by the event sink.
    Sink(E),
}

impl<E> From<Error> for ParseError<E> {
    #[inline]
    fn from(error: Error) -> Self {
        Self::Parse(error)
    }
}

impl<E> fmt::Display for ParseError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => error.fmt(f),
            Self::Sink(error) => error.fmt(f),
        }
    }
}

/// Parse an Eon document into a borrowed event stream.
pub fn parse<'a, S>(source: &'a str, sink: S) -> core::result::Result<(), ParseError<S::Error>>
where
    S: EventSink<'a>,
{
    parse_with_limit(source, sink, DEFAULT_MAX_DEPTH)
}

/// Parse an Eon document into a borrowed event stream with a custom nesting limit.
pub fn parse_with_limit<'a, S>(
    source: &'a str,
    sink: S,
    max_depth: usize,
) -> core::result::Result<(), ParseError<S::Error>>
where
    S: EventSink<'a>,
{
    Parser::new(source, sink, max_depth).parse_document()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DocumentKind {
    ImplicitMap,
    Value,
    ImplicitList,
}

struct Parser<'a, S> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    max_depth: usize,
    sink: S,
}

impl<'a, S> Parser<'a, S>
where
    S: EventSink<'a>,
{
    fn new(source: &'a str, sink: S, max_depth: usize) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            max_depth,
            sink,
        }
    }

    fn parse_document(mut self) -> core::result::Result<(), ParseError<S::Error>> {
        self.skip_ws_comments()?;

        if self.is_eof() {
            self.check_depth(1)?;
            self.emit(Span::new(0, 0), Event::BeginMap { implicit: true })?;
            self.emit(Span::new(0, 0), Event::EndMap)?;
            return Ok(());
        }

        let document_kind = {
            let mut probe = Parser::new(self.source, NoopSink, self.max_depth);
            probe.skip_ws_comments()?;
            probe.lookahead_document_kind()?
        };

        match document_kind {
            DocumentKind::ImplicitMap => self.parse_implicit_map(1)?,
            DocumentKind::Value => self.parse_value(0)?,
            DocumentKind::ImplicitList => self.parse_implicit_list(1)?,
        }

        self.skip_ws_comments()?;
        if self.is_eof() {
            Ok(())
        } else {
            Err(ParseError::Parse(Error::new(
                self.current_span(),
                ErrorKind::TrailingCharacters,
            )))
        }
    }

    fn lookahead_document_kind(&mut self) -> Result<DocumentKind> {
        if self.is_eof() {
            return Ok(DocumentKind::ImplicitMap);
        }

        self.parse_value(0).map_err(|err| match err {
            ParseError::Parse(error) => error,
            ParseError::Sink(_) => unreachable!("lookahead sink never fails"),
        })?;

        self.skip_ws_comments()?;
        Ok(match self.peek_byte() {
            Some(b':') => DocumentKind::ImplicitMap,
            Some(_) => DocumentKind::ImplicitList,
            None => DocumentKind::Value,
        })
    }

    fn parse_implicit_map(
        &mut self,
        depth: usize,
    ) -> core::result::Result<(), ParseError<S::Error>> {
        self.check_depth(depth)?;
        let span = Span::new(self.pos, self.pos);
        self.emit(span, Event::BeginMap { implicit: true })?;

        loop {
            self.skip_ws_comments()?;
            if self.is_eof() {
                self.emit(Span::new(self.pos, self.pos), Event::EndMap)?;
                return Ok(());
            }

            self.emit(self.current_span(), Event::MapKey)?;
            self.parse_value(depth)?;

            self.skip_ws_comments()?;
            self.expect_byte(b':')?;
            self.emit(Span::new(self.pos - 1, self.pos), Event::MapValue)?;

            self.parse_value(depth)?;
            self.skip_ws_comments()?;
            let _ = self.consume_byte(b',');
        }
    }

    fn parse_implicit_list(
        &mut self,
        depth: usize,
    ) -> core::result::Result<(), ParseError<S::Error>> {
        self.check_depth(depth)?;
        let span = Span::new(self.pos, self.pos);
        self.emit(span, Event::BeginList)?;

        loop {
            self.skip_ws_comments()?;
            if self.is_eof() {
                self.emit(Span::new(self.pos, self.pos), Event::EndList)?;
                return Ok(());
            }

            self.parse_value(depth)?;
            self.skip_ws_comments()?;
            let _ = self.consume_byte(b',');
        }
    }

    fn parse_value(&mut self, depth: usize) -> core::result::Result<(), ParseError<S::Error>> {
        self.check_depth(depth)?;
        self.skip_ws_comments()?;

        let Some(byte) = self.peek_byte() else {
            return Err(ParseError::Parse(Error::new(
                Span::new(self.pos, self.pos),
                ErrorKind::ExpectedValue,
            )));
        };

        match byte {
            b'{' => self.parse_explicit_map(depth + 1),
            b'[' => self.parse_list(depth + 1),
            b'"' | b'\'' => self.parse_string_or_variant(depth),
            b'+' | b'-' | b'.' | b'0'..=b'9' => self.parse_number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.parse_identifier_or_variant(depth),
            _ => Err(ParseError::Parse(Error::new(
                self.current_span(),
                ErrorKind::UnexpectedByte(byte),
            ))),
        }
    }

    fn parse_explicit_map(
        &mut self,
        depth: usize,
    ) -> core::result::Result<(), ParseError<S::Error>> {
        self.check_depth(depth)?;
        let start = self.pos;
        self.pos += 1; // {
        self.emit(
            Span::new(start, start + 1),
            Event::BeginMap { implicit: false },
        )?;

        loop {
            self.skip_ws_comments()?;

            if self.consume_byte(b'}') {
                self.emit(Span::new(self.pos - 1, self.pos), Event::EndMap)?;
                return Ok(());
            }

            if self.is_eof() {
                return Err(ParseError::Parse(Error::new(
                    Span::new(self.pos, self.pos),
                    ErrorKind::ExpectedByte(b'}'),
                )));
            }

            self.emit(self.current_span(), Event::MapKey)?;
            self.parse_value(depth)?;

            self.skip_ws_comments()?;
            self.expect_byte(b':')?;
            self.emit(Span::new(self.pos - 1, self.pos), Event::MapValue)?;

            self.parse_value(depth)?;
            self.skip_ws_comments()?;
            let _ = self.consume_byte(b',');
        }
    }

    fn parse_list(&mut self, depth: usize) -> core::result::Result<(), ParseError<S::Error>> {
        self.check_depth(depth)?;
        let start = self.pos;
        self.pos += 1; // [
        self.emit(Span::new(start, start + 1), Event::BeginList)?;

        loop {
            self.skip_ws_comments()?;

            if self.consume_byte(b']') {
                self.emit(Span::new(self.pos - 1, self.pos), Event::EndList)?;
                return Ok(());
            }

            if self.is_eof() {
                return Err(ParseError::Parse(Error::new(
                    Span::new(self.pos, self.pos),
                    ErrorKind::ExpectedByte(b']'),
                )));
            }

            self.parse_value(depth)?;
            self.skip_ws_comments()?;
            let _ = self.consume_byte(b',');
        }
    }

    fn parse_identifier_or_variant(
        &mut self,
        depth: usize,
    ) -> core::result::Result<(), ParseError<S::Error>> {
        let start = self.pos;
        self.pos += 1;
        while matches!(
            self.peek_byte(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        ) {
            self.pos += 1;
        }
        let ident = &self.source[start..self.pos];

        match ident {
            "null" => self.emit(Span::new(start, self.pos), Event::Scalar(Scalar::Null)),
            "true" => self.emit(
                Span::new(start, self.pos),
                Event::Scalar(Scalar::Bool(true)),
            ),
            "false" => self.emit(
                Span::new(start, self.pos),
                Event::Scalar(Scalar::Bool(false)),
            ),
            _ => {
                self.skip_inline_ws();
                if self.consume_byte(b'(') {
                    self.parse_variant(
                        Span::new(start, self.pos),
                        VariantName::Identifier(ident),
                        depth + 1,
                    )
                } else {
                    self.emit(
                        Span::new(start, self.pos),
                        Event::Scalar(Scalar::Identifier(ident)),
                    )
                }
            }
        }
    }

    fn parse_string_or_variant(
        &mut self,
        depth: usize,
    ) -> core::result::Result<(), ParseError<S::Error>> {
        let (string, span) = self.parse_string_token()?;
        self.skip_inline_ws();

        if self.consume_byte(b'(') {
            self.parse_variant(span, VariantName::String(string), depth + 1)
        } else {
            self.emit(span, Event::Scalar(Scalar::String(string)))
        }
    }

    fn parse_variant(
        &mut self,
        span: Span,
        name: VariantName<'a>,
        depth: usize,
    ) -> core::result::Result<(), ParseError<S::Error>> {
        self.check_depth(depth)?;
        self.emit(span, Event::BeginVariant { name })?;

        loop {
            self.skip_ws_comments()?;
            if self.consume_byte(b')') {
                self.emit(Span::new(self.pos - 1, self.pos), Event::EndVariant)?;
                return Ok(());
            }

            if self.is_eof() {
                return Err(ParseError::Parse(Error::new(
                    Span::new(self.pos, self.pos),
                    ErrorKind::ExpectedByte(b')'),
                )));
            }

            self.parse_value(depth)?;
            self.skip_ws_comments()?;
            let _ = self.consume_byte(b',');
        }
    }

    fn parse_number(&mut self) -> core::result::Result<(), ParseError<S::Error>> {
        let start = self.pos;
        self.pos += 1;
        while matches!(
            self.peek_byte(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'+' | b'-' | b'.' | b'_')
        ) {
            self.pos += 1;
        }

        self.emit(
            Span::new(start, self.pos),
            Event::Scalar(Scalar::Number(&self.source[start..self.pos])),
        )
    }

    fn parse_string_token(&mut self) -> Result<(StringToken<'a>, Span)> {
        let start = self.pos;

        if self.starts_with(br#"""""#) {
            self.pos += 3;
            while !self.is_eof() {
                if self.starts_with(br#"""""#) {
                    self.pos += 3;
                    let span = Span::new(start, self.pos);
                    let raw = &self.source[start..self.pos];
                    validate_source_slice(raw, span)?;
                    return Ok((
                        StringToken {
                            raw,
                            kind: StringKind::MultilineBasic,
                        },
                        span,
                    ));
                }
                self.pos += 1;
            }
        } else if self.starts_with(b"'''") {
            self.pos += 3;
            while !self.is_eof() {
                if self.starts_with(b"'''") {
                    self.pos += 3;
                    let span = Span::new(start, self.pos);
                    let raw = &self.source[start..self.pos];
                    validate_source_slice(raw, span)?;
                    return Ok((
                        StringToken {
                            raw,
                            kind: StringKind::MultilineLiteral,
                        },
                        span,
                    ));
                }
                self.pos += 1;
            }
        } else if self.peek_byte() == Some(b'"') {
            self.pos += 1;
            while let Some(byte) = self.peek_byte() {
                match byte {
                    b'\\' => {
                        self.pos += 1;
                        if self.is_eof() {
                            break;
                        }
                        self.pos += 1;
                    }
                    b'"' => {
                        self.pos += 1;
                        let span = Span::new(start, self.pos);
                        let raw = &self.source[start..self.pos];
                        validate_source_slice(raw, span)?;
                        return Ok((
                            StringToken {
                                raw,
                                kind: StringKind::Basic,
                            },
                            span,
                        ));
                    }
                    b'\n' | b'\r' => break,
                    _ => self.pos += 1,
                }
            }
        } else if self.peek_byte() == Some(b'\'') {
            self.pos += 1;
            while let Some(byte) = self.peek_byte() {
                match byte {
                    b'\'' => {
                        self.pos += 1;
                        let span = Span::new(start, self.pos);
                        let raw = &self.source[start..self.pos];
                        validate_source_slice(raw, span)?;
                        return Ok((
                            StringToken {
                                raw,
                                kind: StringKind::Literal,
                            },
                            span,
                        ));
                    }
                    b'\n' | b'\r' => break,
                    _ => self.pos += 1,
                }
            }
        }

        Err(Error::new(
            Span::new(start, self.pos.min(self.source.len())),
            ErrorKind::UnterminatedString,
        ))
    }

    fn emit(
        &mut self,
        span: Span,
        event: Event<'a>,
    ) -> core::result::Result<(), ParseError<S::Error>> {
        self.sink
            .event(SpannedEvent { span, event })
            .map_err(ParseError::Sink)
    }

    fn check_depth(&self, depth: usize) -> Result {
        if depth > self.max_depth {
            Err(Error::new(
                self.current_span(),
                ErrorKind::NestingLimitExceeded,
            ))
        } else {
            Ok(())
        }
    }

    fn skip_ws_comments(&mut self) -> Result {
        loop {
            while matches!(self.peek_byte(), Some(b' ' | b'\t' | b'\n' | b'\r' | 0x0C)) {
                self.pos += 1;
            }

            if self.starts_with(b"//") {
                let start = self.pos;
                self.pos += 2;
                while let Some(byte) = self.peek_byte() {
                    self.pos += 1;
                    if byte == b'\n' {
                        break;
                    }
                }
                validate_source_slice(&self.source[start..self.pos], Span::new(start, self.pos))?;
                continue;
            }

            return Ok(());
        }
    }

    fn skip_inline_ws(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(Error::new(
                self.current_span(),
                ErrorKind::ExpectedByte(expected),
            ))
        }
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.bytes[self.pos..].starts_with(prefix)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn current_span(&self) -> Span {
        if self.is_eof() {
            Span::new(self.pos, self.pos)
        } else {
            Span::new(self.pos, self.pos + 1)
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }
}

fn validate_source_slice(source: &str, span: Span) -> Result {
    if let Some((offset, chr)) = find_disallowed_invisible_unicode(source) {
        return Err(Error::new(
            Span::new(span.start + offset, span.start + offset + chr.len_utf8()),
            ErrorKind::DisallowedInvisibleUnicode(chr),
        ));
    }

    Ok(())
}

fn find_disallowed_invisible_unicode(source: &str) -> Option<(usize, char)> {
    source.char_indices().find(|&(_, chr)| {
        !matches!(chr, '"' | '\\' | '\'' | '\n' | '\r' | '\t')
            && chr.escape_debug().next() == Some('\\')
    })
}

struct NoopSink;

impl<'a> EventSink<'a> for NoopSink {
    type Error = Infallible;

    #[inline]
    fn event(&mut self, _event: SpannedEvent<'a>) -> core::result::Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{string::String, vec::Vec};

    use super::NoopSink;
    use crate::{
        Error, ErrorKind, Event, EventSink, EventWriter, ParseError, Scalar, Span, SpannedEvent,
        StringKind, VariantName, parse, parse_with_limit,
    };

    #[derive(Default)]
    struct CollectSink<'a> {
        events: Vec<SpannedEvent<'a>>,
    }

    impl<'a> EventSink<'a> for CollectSink<'a> {
        type Error = core::convert::Infallible;

        fn event(&mut self, event: SpannedEvent<'a>) -> core::result::Result<(), Self::Error> {
            self.events.push(event);
            Ok(())
        }
    }

    fn assert_borrowed_slice(source: &str, span: Span, slice: &str) {
        assert_eq!(slice, &source[span.start..span.end]);
        assert_eq!(slice.as_ptr(), source[span.start..].as_ptr());
    }

    #[test]
    fn parse_empty_document_as_implicit_map() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 16>::new(&mut out);
        parse("", &mut writer).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "");
    }

    #[test]
    fn parse_unquoted_unit_variant_value() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 16>::new(&mut out);
        parse("some_enum: EnumValue", &mut writer).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "some_enum: EnumValue");
    }

    #[test]
    fn parse_payload_variant_and_list() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 32>::new(&mut out);
        parse(
            "some_enum: EnumValue(42, \"hello\")\nlist: [true false]",
            &mut writer,
        )
        .unwrap();
        writer.finish().unwrap();
        assert_eq!(
            out,
            "some_enum: EnumValue(42, \"hello\"), list: [true, false]"
        );
    }

    #[test]
    fn quoted_value_stays_a_string() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 16>::new(&mut out);
        parse("some_enum: \"EnumValue\"", &mut writer).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "some_enum: \"EnumValue\"");
    }

    #[test]
    fn quoted_variant_name_is_supported() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 16>::new(&mut out);
        parse("some_enum: \"kebab-case\"()", &mut writer).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "some_enum: \"kebab-case\"()");
    }

    #[test]
    fn parse_top_level_implicit_list() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 16>::new(&mut out);
        parse("1, 2 3", &mut writer).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "[1, 2, 3]");
    }

    #[test]
    fn parse_root_implicit_map_with_explicit_map_key() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 32>::new(&mut out);
        parse("{ nested: true }: answer", &mut writer).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "{nested: true}: answer");
    }

    #[test]
    fn parser_borrows_identifier_and_number_tokens_from_source() {
        let source = "mode: EnumValue(42)";
        let mut sink = CollectSink::default();
        parse(source, &mut sink).unwrap();

        let (identifier_span, identifier) = sink
            .events
            .iter()
            .find_map(|event| match event.event {
                Event::Scalar(Scalar::Identifier(identifier)) if identifier == "mode" => {
                    Some((event.span, identifier))
                }
                _ => None,
            })
            .unwrap();
        assert_borrowed_slice(source, identifier_span, identifier);

        let (number_span, number) = sink
            .events
            .iter()
            .find_map(|event| match event.event {
                Event::Scalar(Scalar::Number(number)) => Some((event.span, number)),
                _ => None,
            })
            .unwrap();
        assert_eq!(number, "42");
        assert_borrowed_slice(source, number_span, number);
    }

    #[test]
    fn parser_borrows_raw_escaped_string_tokens_from_source() {
        let source = "label: \"he\\nllo\"";
        let mut sink = CollectSink::default();
        parse(source, &mut sink).unwrap();

        let (string_span, raw, kind) = sink
            .events
            .iter()
            .find_map(|event| match event.event {
                Event::Scalar(Scalar::String(token)) => Some((event.span, token.raw, token.kind)),
                _ => None,
            })
            .unwrap();
        assert_eq!(kind, StringKind::Basic);
        assert_eq!(raw, "\"he\\nllo\"");
        assert_borrowed_slice(source, string_span, raw);
    }

    #[test]
    fn parser_borrows_quoted_variant_heads_from_source() {
        let source = "mode: \"kebab-case\"()";
        let mut sink = CollectSink::default();
        parse(source, &mut sink).unwrap();

        let (variant_span, raw, kind) = sink
            .events
            .iter()
            .find_map(|event| match event.event {
                Event::BeginVariant {
                    name: VariantName::String(token),
                } => Some((event.span, token.raw, token.kind)),
                _ => None,
            })
            .unwrap();
        assert_eq!(kind, StringKind::Basic);
        assert_eq!(raw, "\"kebab-case\"");
        assert_borrowed_slice(source, variant_span, raw);
    }

    #[test]
    fn depth_limit_counts_root_containers_once() {
        parse_with_limit("[1]", NoopSink, 1).unwrap();
        parse_with_limit("{answer: 42}", NoopSink, 1).unwrap();
        parse_with_limit("answer: 42", NoopSink, 1).unwrap();
        parse_with_limit("EnumValue()", NoopSink, 1).unwrap();
        parse_with_limit("1, 2, 3", NoopSink, 1).unwrap();
        parse_with_limit("{nested: true}: answer", NoopSink, 2).unwrap();
    }

    #[test]
    fn depth_limit_rejects_nested_containers() {
        let err = parse_with_limit("[[]]", NoopSink, 1).unwrap_err();
        assert!(matches!(
            err,
            ParseError::Parse(Error {
                kind: ErrorKind::NestingLimitExceeded,
                ..
            })
        ));

        let err = parse_with_limit("answer: []", NoopSink, 1).unwrap_err();
        assert!(matches!(
            err,
            ParseError::Parse(Error {
                kind: ErrorKind::NestingLimitExceeded,
                ..
            })
        ));
    }

    #[test]
    fn empty_document_respects_depth_limit() {
        let err = parse_with_limit("", NoopSink, 0).unwrap_err();
        assert!(matches!(
            err,
            ParseError::Parse(Error {
                kind: ErrorKind::NestingLimitExceeded,
                ..
            })
        ));
    }
}
