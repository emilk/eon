use alloc::vec::Vec;

use crate::{
    Comment, Document, Error, Item, KeyValue, Map, ParseErrorKind, Result, Span, Token, TokenKind,
    TokenStream, Trivia, TriviaKind, Value, ValueTree, Variant, VariantName, lex,
};

/// Default maximum nesting depth for formatter-core parsing.
pub const DEFAULT_MAX_DEPTH: usize = 128;

#[derive(Debug)]
struct PreparedDocument<'a> {
    tokens: Vec<PreparedToken<'a>>,
    trailing_comments: Vec<Comment<'a>>,
    trailing_suffix_of_last: Option<Comment<'a>>,
    end_offset: usize,
}

#[derive(Clone, Debug)]
struct PreparedToken<'a> {
    token: Token<'a>,
    prefix_comments: Vec<Comment<'a>>,
    suffix_of_previous: Option<Comment<'a>>,
}

#[derive(Clone)]
struct Parser<'a, 'd> {
    doc: &'d PreparedDocument<'a>,
    index: usize,
    max_depth: usize,
}

/// Parse an Eon document into a formatter-oriented syntax tree.
pub fn parse_document(source: &str) -> Result<Document<'_>> {
    parse_document_with_limit(source, DEFAULT_MAX_DEPTH)
}

fn parse_document_with_limit(source: &str, max_depth: usize) -> Result<Document<'_>> {
    let stream = lex(source)?;
    let prepared = prepare_document(&stream, source.len());

    let mut map_parser = Parser::new(&prepared, max_depth);
    let implicit_map = map_parser.parse_implicit_root_map();

    if let Ok(document) = implicit_map {
        return Ok(document);
    }

    let mut list_parser = Parser::new(&prepared, max_depth);
    let list_or_value = list_parser.parse_root_list_or_value();

    match (implicit_map, list_or_value) {
        (Err(map_err), Ok(document)) => {
            let _ = map_err;
            Ok(document)
        }
        (Err(map_err), Err(list_err)) => {
            if list_err.span.start >= map_err.span.start {
                Err(list_err)
            } else {
                Err(map_err)
            }
        }
        (Ok(document), _) => Ok(document),
    }
}

fn prepare_document<'a>(stream: &TokenStream<'a>, end_offset: usize) -> PreparedDocument<'a> {
    let mut tokens = Vec::new();
    let mut pending_trivia = Vec::new();

    for item in &stream.items {
        match item {
            Item::Token(token) => {
                let (suffix, prefix) = split_comments(&pending_trivia, !tokens.is_empty());
                tokens.push(PreparedToken {
                    token: *token,
                    prefix_comments: prefix,
                    suffix_of_previous: suffix,
                });
                pending_trivia.clear();
            }
            Item::Trivia(trivia) => pending_trivia.push(*trivia),
        }
    }

    let (trailing_suffix_of_last, trailing_comments) =
        split_comments(&pending_trivia, !tokens.is_empty());

    PreparedDocument {
        tokens,
        trailing_comments,
        trailing_suffix_of_last,
        end_offset,
    }
}

fn split_comments<'a>(
    trivia: &[Trivia<'a>],
    can_have_suffix: bool,
) -> (Option<Comment<'a>>, Vec<Comment<'a>>) {
    let mut saw_line_break = false;
    let mut suffix = None;
    let mut prefix = Vec::new();

    for piece in trivia {
        match piece.kind {
            TriviaKind::Whitespace => {
                if piece.line_breaks > 0 {
                    saw_line_break = true;
                }
            }
            TriviaKind::Comment => {
                if can_have_suffix && !saw_line_break && suffix.is_none() {
                    suffix = Some(piece.raw);
                } else {
                    prefix.push(piece.raw);
                }
            }
        }
    }

    (suffix, prefix)
}

impl<'a, 'd> Parser<'a, 'd> {
    fn new(doc: &'d PreparedDocument<'a>, max_depth: usize) -> Self {
        Self {
            doc,
            index: 0,
            max_depth,
        }
    }

    fn parse_implicit_root_map(&mut self) -> Result<Document<'a>> {
        let map = self.parse_map_body(None, 1)?;
        if !self.is_eof() {
            return Err(self.parse_error_current(ParseErrorKind::TrailingTokens));
        }

        Ok(Document {
            root: Value::Map(map).into(),
            implicit_root_map: true,
            trailing_comments: Vec::new(),
        })
    }

    fn parse_root_list_or_value(&mut self) -> Result<Document<'a>> {
        let mut value_parser = self.clone();
        let root = value_parser.parse_value(0)?;
        if value_parser.is_eof() {
            Ok(Document {
                root,
                implicit_root_map: false,
                trailing_comments: value_parser.current_prefix_comments(),
            })
        } else {
            let list = self.parse_list_body(None, 1)?;
            if !self.is_eof() {
                return Err(self.parse_error_current(ParseErrorKind::TrailingTokens));
            }

            Ok(Document {
                root: Value::List(list).into(),
                implicit_root_map: false,
                trailing_comments: Vec::new(),
            })
        }
    }

    fn parse_map_body(&mut self, closing: Option<TokenKind>, depth: usize) -> Result<Map<'a>> {
        let mut key_values = Vec::new();

        loop {
            if self.at_end_of_body(closing) {
                return Ok(Map {
                    key_values,
                    closing_comments: self.current_prefix_comments(),
                });
            }

            let key = self.parse_value(depth)?;
            self.consume_clean(TokenKind::Colon)?;

            let mut value = self.parse_value(depth)?;
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.consume_clean(TokenKind::Comma)?;
                value.suffix_comment = self.peek_suffix_of_previous();
            }

            key_values.push(KeyValue { key, value });
        }
    }

    fn parse_list_body(
        &mut self,
        closing: Option<TokenKind>,
        depth: usize,
    ) -> Result<crate::List<'a>> {
        let mut values = Vec::new();

        loop {
            if self.at_end_of_body(closing) {
                return Ok(crate::List {
                    values,
                    closing_comments: self.current_prefix_comments(),
                });
            }

            let mut value = self.parse_value(depth)?;
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.consume_clean(TokenKind::Comma)?;
                value.suffix_comment = self.peek_suffix_of_previous();
            }
            values.push(value);
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<ValueTree<'a>> {
        if depth > self.max_depth {
            return Err(self.parse_error_current(ParseErrorKind::MaxDepthExceeded));
        }

        let prefix_comments = self.current_prefix_comments();

        let Some(prepared) = self.next_token() else {
            return Err(self.parse_error_current(ParseErrorKind::ExpectedValue));
        };

        let value = match prepared.token.kind {
            TokenKind::OpenBrace => {
                let map = self.parse_map_body(Some(TokenKind::CloseBrace), depth + 1)?;
                self.consume_closer(TokenKind::CloseBrace)?;
                Value::Map(map)
            }
            TokenKind::OpenList => {
                let list = self.parse_list_body(Some(TokenKind::CloseList), depth + 1)?;
                self.consume_closer(TokenKind::CloseList)?;
                Value::List(list)
            }
            TokenKind::Identifier => {
                if self.peek_kind() == Some(TokenKind::OpenParen) && self.peek_is_clean() {
                    self.consume_clean(TokenKind::OpenParen)?;
                    let list = self.parse_list_body(Some(TokenKind::CloseParen), depth + 1)?;
                    self.consume_closer(TokenKind::CloseParen)?;
                    Value::Variant(Variant {
                        name: VariantName::Identifier(prepared.token.raw),
                        values: list.values,
                        closing_comments: list.closing_comments,
                    })
                } else {
                    Value::Identifier(prepared.token.raw)
                }
            }
            TokenKind::String(_) => {
                if self.peek_kind() == Some(TokenKind::OpenParen) && self.peek_is_clean() {
                    self.consume_clean(TokenKind::OpenParen)?;
                    let list = self.parse_list_body(Some(TokenKind::CloseParen), depth + 1)?;
                    self.consume_closer(TokenKind::CloseParen)?;
                    Value::Variant(Variant {
                        name: VariantName::Quoted(prepared.token.raw),
                        values: list.values,
                        closing_comments: list.closing_comments,
                    })
                } else {
                    Value::QuotedString(prepared.token.raw)
                }
            }
            TokenKind::Number => Value::Number(prepared.token.raw),
            TokenKind::CloseBrace | TokenKind::CloseList | TokenKind::CloseParen => {
                return Err(self.parse_error(
                    prepared.token.span,
                    ParseErrorKind::Unbalanced(prepared.token.kind),
                ));
            }
            TokenKind::OpenParen | TokenKind::Colon | TokenKind::Comma => {
                return Err(self.parse_error(
                    prepared.token.span,
                    ParseErrorKind::UnexpectedToken(prepared.token.kind),
                ));
            }
        };

        Ok(ValueTree {
            prefix_comments,
            value,
            suffix_comment: self.peek_suffix_of_previous(),
        })
    }

    fn at_end_of_body(&self, closing: Option<TokenKind>) -> bool {
        match (closing, self.peek_kind()) {
            (Some(kind), Some(next)) => next == kind,
            (None, None) => true,
            _ => false,
        }
    }

    fn consume_clean(&mut self, kind: TokenKind) -> Result<()> {
        let Some(prepared) = self.peek() else {
            return Err(self.parse_error_current(ParseErrorKind::ExpectedToken(kind)));
        };

        if prepared.token.kind != kind {
            return Err(self.parse_error(prepared.token.span, ParseErrorKind::ExpectedToken(kind)));
        }

        if prepared.suffix_of_previous.is_some() || !prepared.prefix_comments.is_empty() {
            return Err(
                self.parse_error(prepared.token.span, ParseErrorKind::UnexpectedToken(kind))
            );
        }

        self.index += 1;
        Ok(())
    }

    fn consume_closer(&mut self, kind: TokenKind) -> Result<()> {
        let Some(prepared) = self.peek() else {
            return Err(self.parse_error_current(ParseErrorKind::ExpectedToken(kind)));
        };

        if prepared.token.kind != kind {
            return Err(self.parse_error(prepared.token.span, ParseErrorKind::ExpectedToken(kind)));
        }

        self.index += 1;
        Ok(())
    }

    fn peek(&self) -> Option<&PreparedToken<'a>> {
        self.doc.tokens.get(self.index)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|prepared| prepared.token.kind)
    }

    fn peek_is_clean(&self) -> bool {
        self.peek().is_some_and(|prepared| {
            prepared.suffix_of_previous.is_none() && prepared.prefix_comments.is_empty()
        })
    }

    fn next_token(&mut self) -> Option<PreparedToken<'a>> {
        let prepared = self.doc.tokens.get(self.index)?.clone();
        self.index += 1;
        Some(prepared)
    }

    fn current_prefix_comments(&self) -> Vec<Comment<'a>> {
        if let Some(prepared) = self.peek() {
            prepared.prefix_comments.clone()
        } else {
            self.doc.trailing_comments.clone()
        }
    }

    fn peek_suffix_of_previous(&self) -> Option<Comment<'a>> {
        if let Some(prepared) = self.peek() {
            prepared.suffix_of_previous
        } else {
            self.doc.trailing_suffix_of_last
        }
    }

    fn is_eof(&self) -> bool {
        self.index >= self.doc.tokens.len()
    }

    fn parse_error_current(&self, kind: ParseErrorKind) -> Error {
        self.parse_error(self.current_span(), kind)
    }

    fn parse_error(&self, span: Span, kind: ParseErrorKind) -> Error {
        Error::parse(span, kind)
    }

    fn current_span(&self) -> Span {
        self.peek()
            .map(|prepared| prepared.token.span)
            .unwrap_or_else(|| Span::new(self.doc.end_offset, self.doc.end_offset))
    }
}

#[cfg(test)]
mod tests {
    use std::string::ToString;

    use super::{parse_document, parse_document_with_limit};
    use crate::{ParseErrorKind, Value, VariantName};

    #[test]
    fn parse_comments_and_suffix_comments() {
        let document = parse_document("// prefix\nkey: true // suffix\n").unwrap();
        let Value::Map(map) = &document.root.value else {
            panic!("expected root map");
        };

        assert!(document.trailing_comments.is_empty());
        assert_eq!(map.key_values.len(), 1);
        assert_eq!(
            map.key_values[0].key.prefix_comments,
            alloc::vec!["// prefix"]
        );
        assert_eq!(map.key_values[0].value.suffix_comment, Some("// suffix"));
    }

    #[test]
    fn parse_variant_names_from_identifiers_and_strings() {
        let document = parse_document("a: EnumValue(1)\nb: \"Quoted\"(2)").unwrap();
        let Value::Map(map) = &document.root.value else {
            panic!("expected root map");
        };

        let Value::Variant(first) = &map.key_values[0].value.value else {
            panic!("expected identifier variant");
        };
        assert_eq!(first.name, VariantName::Identifier("EnumValue"));

        let Value::Variant(second) = &map.key_values[1].value.value else {
            panic!("expected quoted variant");
        };
        assert_eq!(second.name, VariantName::Quoted("\"Quoted\""));
    }

    #[test]
    fn parse_comments_before_variant_payload_are_rejected() {
        let err = parse_document("\"Rgb\" // nope\n({ r: 1 })").unwrap_err();
        assert!(err.to_string().contains("unexpected '('"));
    }

    #[test]
    fn parse_single_root_value_keeps_trailing_line_comments() {
        let document = parse_document("1\n// tail\n").unwrap();

        assert_eq!(document.root.value, Value::Number("1"));
        assert_eq!(document.trailing_comments, alloc::vec!["// tail"]);
    }

    #[test]
    fn depth_limit_counts_root_maps_once() {
        parse_document_with_limit("outer: { inner: { leaf: 1 } }", 3).unwrap();
        let err = parse_document_with_limit("outer: { inner: { leaf: 1 } }", 2).unwrap_err();
        assert_eq!(
            err.kind,
            crate::ErrorKind::Parse(ParseErrorKind::MaxDepthExceeded)
        );

        parse_document_with_limit("{ outer: { inner: { leaf: 1 } } }", 3).unwrap();
        let err = parse_document_with_limit("{ outer: { inner: { leaf: 1 } } }", 2).unwrap_err();
        assert_eq!(
            err.kind,
            crate::ErrorKind::Parse(ParseErrorKind::MaxDepthExceeded)
        );
    }

    #[test]
    fn depth_limit_counts_root_lists_once() {
        parse_document_with_limit("[[1]]", 2).unwrap();
        let err = parse_document_with_limit("[[1]]", 1).unwrap_err();
        assert_eq!(
            err.kind,
            crate::ErrorKind::Parse(ParseErrorKind::MaxDepthExceeded)
        );

        parse_document_with_limit("1, [2]", 2).unwrap();
        let err = parse_document_with_limit("1, [2]", 1).unwrap_err();
        assert_eq!(
            err.kind,
            crate::ErrorKind::Parse(ParseErrorKind::MaxDepthExceeded)
        );
    }
}
