use alloc::vec::Vec;

use crate::{Error, ErrorKind, Result, Span, Token, TokenKind, TokenStream, TriviaKind};

/// Top-level document shapes supported by the formatter stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentKind {
    /// Brace-less root map such as `alpha: 1`.
    ImplicitMap,
    /// A single root value such as `42` or `{ alpha: 1 }`.
    Value,
    /// A brace-less root list such as `1, 2, 3`.
    ImplicitList,
}

/// A token span covering one syntactic value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueSpan {
    /// Inclusive start token index.
    pub start: usize,
    /// Inclusive end token index.
    pub end: usize,
}

impl ValueSpan {
    /// Create a new token span.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// One top-level implicit map entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapEntry {
    /// Key token span.
    pub key: ValueSpan,
    /// Colon token index.
    pub colon: usize,
    /// Value token span.
    pub value: ValueSpan,
}

/// An analyzed root document over a borrowed token stream.
#[derive(Debug)]
pub struct Document<'stream, 'source> {
    stream: &'stream TokenStream<'source>,
    kind: DocumentKind,
    root: Option<ValueSpan>,
    values: Vec<ValueSpan>,
    entries: Vec<MapEntry>,
}

impl<'stream, 'source> Document<'stream, 'source> {
    /// The root document shape.
    pub fn kind(&self) -> DocumentKind {
        self.kind
    }

    /// The underlying borrowed token stream.
    pub fn stream(&self) -> &'stream TokenStream<'source> {
        self.stream
    }

    /// The single root value when the document kind is [`DocumentKind::Value`].
    pub fn root_value(&self) -> Option<ValueSpan> {
        self.root
    }

    /// Top-level implicit-list items.
    pub fn values(&self) -> &[ValueSpan] {
        &self.values
    }

    /// Top-level implicit-map entries.
    pub fn entries(&self) -> &[MapEntry] {
        &self.entries
    }
}

/// Analyze the root shape of a borrowed token stream.
pub fn analyze_document<'stream, 'source>(
    stream: &'stream TokenStream<'source>,
) -> Result<Document<'stream, 'source>> {
    Analyzer { stream }.analyze_document()
}

struct Analyzer<'stream, 'source> {
    stream: &'stream TokenStream<'source>,
}

impl<'stream, 'source> Analyzer<'stream, 'source> {
    fn analyze_document(self) -> Result<Document<'stream, 'source>> {
        if self.stream.has_no_tokens() {
            return Ok(Document {
                stream: self.stream,
                kind: DocumentKind::ImplicitMap,
                root: None,
                values: Vec::new(),
                entries: Vec::new(),
            });
        }

        let first = self.parse_value(0)?;
        let next = first.end + 1;

        match self.stream.token(next) {
            Some(Token {
                kind: TokenKind::Colon,
                ..
            }) => {
                let entries = self.parse_implicit_map()?;
                Ok(Document {
                    stream: self.stream,
                    kind: DocumentKind::ImplicitMap,
                    root: None,
                    values: Vec::new(),
                    entries,
                })
            }
            Some(_) => {
                let values = self.parse_implicit_list()?;
                if values.len() == 1 {
                    Ok(Document {
                        stream: self.stream,
                        kind: DocumentKind::Value,
                        root: values.first().copied(),
                        values: Vec::new(),
                        entries: Vec::new(),
                    })
                } else {
                    Ok(Document {
                        stream: self.stream,
                        kind: DocumentKind::ImplicitList,
                        root: None,
                        values,
                        entries: Vec::new(),
                    })
                }
            }
            None => Ok(Document {
                stream: self.stream,
                kind: DocumentKind::Value,
                root: Some(first),
                values: Vec::new(),
                entries: Vec::new(),
            }),
        }
    }

    fn parse_implicit_map(&self) -> Result<Vec<MapEntry>> {
        let mut entries = Vec::new();
        let mut next = 0;

        while next < self.stream.token_count() {
            let key = self.parse_value(next)?;
            let colon = key.end + 1;
            let Some(token) = self.stream.token(colon) else {
                return Err(self.error_at_end(ErrorKind::UnexpectedEnd));
            };
            if token.kind != TokenKind::Colon {
                return Err(self.error_at_token(
                    colon,
                    ErrorKind::MalformedStructure("expected `:` after implicit map key"),
                ));
            }

            let value_start = colon + 1;
            let value = self.parse_value(value_start)?;
            entries.push(MapEntry { key, colon, value });

            next = value.end + 1;
            if matches!(
                self.stream.token(next),
                Some(Token {
                    kind: TokenKind::Comma,
                    ..
                })
            ) {
                next += 1;
            }
        }

        Ok(entries)
    }

    fn parse_implicit_list(&self) -> Result<Vec<ValueSpan>> {
        let mut values = Vec::new();
        let mut next = 0;

        while next < self.stream.token_count() {
            let value = self.parse_value(next)?;
            values.push(value);

            next = value.end + 1;
            if matches!(
                self.stream.token(next),
                Some(Token {
                    kind: TokenKind::Comma,
                    ..
                })
            ) {
                next += 1;
            }
        }

        Ok(values)
    }

    fn parse_value(&self, start: usize) -> Result<ValueSpan> {
        let Some(token) = self.stream.token(start) else {
            return Err(self.error_at_end(ErrorKind::UnexpectedEnd));
        };

        match token.kind {
            TokenKind::OpenList | TokenKind::OpenBrace => self
                .scan_balanced_value(start)
                .map(|end| ValueSpan::new(start, end)),
            TokenKind::Identifier | TokenKind::String(_) => {
                if self.is_inline_variant_start(start) {
                    self.scan_balanced_value(start + 1)
                        .map(|end| ValueSpan::new(start, end))
                } else {
                    Ok(ValueSpan::new(start, start))
                }
            }
            TokenKind::Number => Ok(ValueSpan::new(start, start)),
            TokenKind::CloseList
            | TokenKind::CloseBrace
            | TokenKind::CloseParen
            | TokenKind::Colon
            | TokenKind::Comma
            | TokenKind::OpenParen => Err(self.error_at_token(
                start,
                ErrorKind::MalformedStructure("unexpected token while parsing a value"),
            )),
        }
    }

    fn is_inline_variant_start(&self, start: usize) -> bool {
        let Some(name) = self.stream.token_ref(start) else {
            return false;
        };
        let Some(next) = self.stream.token(start + 1) else {
            return false;
        };

        if next.kind != TokenKind::OpenParen {
            return false;
        }

        name.suffix_comment().is_none()
            && name
                .trailing_trivia()
                .all(|trivia| trivia.kind == TriviaKind::Whitespace && trivia.line_breaks == 0)
    }

    fn scan_balanced_value(&self, start_open: usize) -> Result<usize> {
        let mut expected_closers = Vec::new();
        expected_closers.push(
            matching_close(
                self.stream
                    .token(start_open)
                    .expect("scan_balanced_value starts on an existing token")
                    .kind,
            )
            .ok_or(self.error_at_token(
                start_open,
                ErrorKind::MalformedStructure("expected an opening delimiter"),
            ))?,
        );

        let mut next = start_open + 1;
        while let Some(token) = self.stream.token(next) {
            match token.kind {
                TokenKind::OpenList | TokenKind::OpenBrace | TokenKind::OpenParen => {
                    expected_closers.push(
                        matching_close(token.kind)
                            .expect("opening delimiter must have a matching close delimiter"),
                    );
                }
                TokenKind::CloseList | TokenKind::CloseBrace | TokenKind::CloseParen => {
                    let Some(expected) = expected_closers.pop() else {
                        return Err(self.error_at_token(
                            next,
                            ErrorKind::MalformedStructure("unexpected closing delimiter"),
                        ));
                    };
                    if token.kind != expected {
                        return Err(self.error_at_token(
                            next,
                            ErrorKind::MalformedStructure("mismatched closing delimiter"),
                        ));
                    }
                    if expected_closers.is_empty() {
                        return Ok(next);
                    }
                }
                TokenKind::Colon
                | TokenKind::Comma
                | TokenKind::Identifier
                | TokenKind::Number
                | TokenKind::String(_) => {}
            }

            next += 1;
        }

        Err(self.error_at_end(ErrorKind::UnexpectedEnd))
    }

    fn error_at_token(&self, token_index: usize, kind: ErrorKind) -> Error {
        let token = self
            .stream
            .token(token_index)
            .expect("error_at_token requires an existing token index");
        Error::new(token.span, kind)
    }

    fn error_at_end(&self, kind: ErrorKind) -> Error {
        let end = self
            .stream
            .items
            .last()
            .map(|item| match item {
                crate::Item::Token(token) => token.span.end,
                crate::Item::Trivia(trivia) => trivia.span.end,
            })
            .unwrap_or(0);
        Error::new(Span::new(end, end), kind)
    }
}

fn matching_close(kind: TokenKind) -> Option<TokenKind> {
    match kind {
        TokenKind::OpenList => Some(TokenKind::CloseList),
        TokenKind::OpenBrace => Some(TokenKind::CloseBrace),
        TokenKind::OpenParen => Some(TokenKind::CloseParen),
        TokenKind::CloseList
        | TokenKind::CloseBrace
        | TokenKind::CloseParen
        | TokenKind::Colon
        | TokenKind::Comma
        | TokenKind::Identifier
        | TokenKind::Number
        | TokenKind::String(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use std::vec;

    use crate::{DocumentKind, ErrorKind, analyze_document, lex};

    #[test]
    fn empty_stream_is_an_implicit_map() {
        let stream = lex("").unwrap();
        let document = analyze_document(&stream).unwrap();

        assert_eq!(document.kind(), DocumentKind::ImplicitMap);
        assert!(document.entries().is_empty());
    }

    #[test]
    fn detects_root_implicit_map_entries() {
        let stream = lex("alpha: 1, beta: [2]").unwrap();
        let document = analyze_document(&stream).unwrap();

        assert_eq!(document.kind(), DocumentKind::ImplicitMap);
        assert_eq!(
            entry_token_texts(&stream, document.entries()),
            vec![("alpha", "1"), ("beta", "[2]"),]
        );
    }

    #[test]
    fn detects_root_implicit_list_items() {
        let stream = lex("1, 2 3").unwrap();
        let document = analyze_document(&stream).unwrap();

        assert_eq!(document.kind(), DocumentKind::ImplicitList);
        assert_eq!(
            value_token_texts(&stream, document.values()),
            vec!["1", "2", "3"]
        );
    }

    #[test]
    fn detects_single_root_value() {
        let stream = lex("Rgb(1)").unwrap();
        let document = analyze_document(&stream).unwrap();

        assert_eq!(document.kind(), DocumentKind::Value);
        let root = document.root_value().unwrap();
        assert_eq!(span_text(&stream, root), "Rgb(1)");
    }

    #[test]
    fn explicit_map_key_can_start_an_implicit_root_map() {
        let stream = lex("{ nested: true }: answer").unwrap();
        let document = analyze_document(&stream).unwrap();

        assert_eq!(document.kind(), DocumentKind::ImplicitMap);
        assert_eq!(
            entry_token_texts(&stream, document.entries()),
            vec![("{ nested: true }", "answer"),]
        );
    }

    #[test]
    fn malformed_root_map_reports_error() {
        let stream = lex("alpha:").unwrap();
        let err = analyze_document(&stream).unwrap_err();

        assert_eq!(err.kind, ErrorKind::UnexpectedEnd);
    }

    #[test]
    fn variant_requires_inline_trivia_before_open_paren() {
        let stream = lex("Rgb\n(1)").unwrap();
        let err = analyze_document(&stream).unwrap_err();

        assert_eq!(
            err.kind,
            ErrorKind::MalformedStructure("unexpected token while parsing a value")
        );
    }

    fn entry_token_texts<'a>(
        stream: &'a crate::TokenStream<'a>,
        entries: &[crate::MapEntry],
    ) -> Vec<(&'a str, &'a str)> {
        entries
            .iter()
            .map(|entry| (span_text(stream, entry.key), span_text(stream, entry.value)))
            .collect()
    }

    fn value_token_texts<'a>(
        stream: &'a crate::TokenStream<'a>,
        values: &[crate::ValueSpan],
    ) -> Vec<&'a str> {
        values
            .iter()
            .map(|value| span_text(stream, *value))
            .collect()
    }

    fn span_text<'a>(stream: &'a crate::TokenStream<'a>, span: crate::ValueSpan) -> &'a str {
        let start = stream.token(span.start).unwrap().span.start;
        let end = stream.token(span.end).unwrap().span.end;
        &stream.source[start..end]
    }
}
