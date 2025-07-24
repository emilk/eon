use crate::{
    ast::{CommentedValue, KeyValue, List, Object, PlacedToken, Value},
    error::{Error, ErrorReport, Result, error_report_at},
    span::Span,
    token::TokenKind,
};

pub struct PlacedTokenResult<'s> {
    /// The span of the token in the input string.
    pub span: Span,

    /// The token value
    pub slice: &'s str,

    /// The token type
    pub kind: Result<TokenKind>,
}

impl<'s> PlacedTokenResult<'s> {
    /// Returns the inner token, or an error if the token is invalid.
    pub fn ok(self) -> Result<PlacedToken<'s>> {
        match self.kind {
            Ok(token) => Ok(PlacedToken {
                span: self.span,
                slice: self.slice,
                kind: token,
            }),
            Err(err) => Err(err),
        }
    }
}

pub struct PlacedTokenIter<'s> {
    iter: logos::SpannedIter<'s, TokenKind>,
}

impl<'s> Iterator for PlacedTokenIter<'s> {
    type Item = PlacedTokenResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, span) = self.iter.next()?;
        let span = Span {
            start: span.start,
            end: span.end,
        };
        let slice = self.iter.slice();
        if let Ok(token) = result {
            Some(PlacedTokenResult {
                span,
                slice,
                kind: Ok(token),
            })
        } else {
            let report = error_report_at(span, format!("Invalid token: '{slice}'"));
            Some(PlacedTokenResult {
                span,
                slice,
                kind: Err(Error::new(self.iter.source(), report)),
            })
        }
    }
}

pub struct PeekableIter<'s> {
    source: &'s str,
    iter: PlacedTokenIter<'s>,

    /// Remember a peeked value, even if it was None.
    #[expect(clippy::option_option)]
    peeked: Option<Option<PlacedTokenResult<'s>>>,

    last_span: Span,
}

impl<'s> PeekableIter<'s> {
    pub fn new(source: &'s str) -> Self {
        use logos::Logos as _;
        PeekableIter {
            source,
            iter: PlacedTokenIter {
                iter: TokenKind::lexer(source).spanned(),
            },
            peeked: None,
            last_span: Span { start: 0, end: 0 },
        }
    }

    pub fn source(&self) -> &'s str {
        self.source
    }

    pub fn error(&self, report: ErrorReport) -> Error {
        Error::new(self.source, report)
    }

    pub fn error_at(&self, span: Span, message: impl Into<String>) -> Error {
        self.error(error_report_at(span, message))
    }

    pub fn peek(&mut self) -> Option<&PlacedTokenResult<'s>> {
        let iter = &mut self.iter;
        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    /// Span of the next token returned by [`Self::peek()`].
    ///
    /// If there is no next token, will return [`Self::span_of_previous`].
    pub fn span_of_next(&self) -> Span {
        self.peeked
            .as_ref()
            .and_then(|v| v.as_ref())
            .map_or(self.last_span, |v| v.span)
    }

    /// Span of the latest token returned by [`Self::next()`].
    pub fn span_of_previous(&self) -> Span {
        self.last_span
    }

    pub fn span_of_end(&self) -> Span {
        Span {
            start: self.last_span.end,
            end: self.source.len(),
        }
    }
}

impl<'s> Iterator for PeekableIter<'s> {
    type Item = PlacedTokenResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next(),
        };
        if let Some(next) = &next {
            self.last_span = next.span;
        }
        next
    }
}

/// Parse a full Con file.
pub fn parse_top_str(source: &str) -> Result<CommentedValue<'_>> {
    // Usually a Con file contains a bunch of `key: value` pairs, without any
    // surrounding braces, so we optimize for that case:
    let mut tokens_a = PeekableIter::new(source);
    match parse_object_contents(&mut tokens_a) {
        Ok(object) => {
            check_for_trailing_tokens(&mut tokens_a)?;
            let value = CommentedValue {
                span: Span {
                    start: 0,
                    end: source.len(),
                },
                prefix_comments: vec![],
                value: Value::Object(object),
                suffix_comment: None,
            };
            Ok(value)
        }
        Err(err_a) => {
            // Maybe the use did wrap the file in {}, or maybe it is not an object?
            let mut tokens_b = PeekableIter::new(source);

            match parse_commented_value(&mut tokens_b) {
                Ok(value) => {
                    check_for_trailing_tokens(&mut tokens_b)?;
                    Ok(value)
                }
                Err(err_b) => {
                    // Return the error of the path that processed the most tokens, i.e. got further:
                    if tokens_a.span_of_previous().end < tokens_b.span_of_previous().end {
                        Err(err_b)
                    } else {
                        Err(err_a)
                    }
                }
            }
        }
    }
}

fn check_for_trailing_tokens(tokens: &mut PeekableIter<'_>) -> Result {
    if let Some(token) = tokens.next() {
        let token = token.ok()?;
        Err(tokens.error_at(token.span, "Expected end of file here"))
    } else {
        Ok(())
    }
}

/// Parse the inside of a list, without consuming either the opening or closing brackets.
fn parse_list_contents<'s>(tokens: &mut PeekableIter<'s>) -> Result<List<'s>> {
    let mut values = vec![];

    loop {
        let prefix_comments = parse_comments(tokens);

        if tokens.peek().is_none_or(|peeked| {
            matches!(
                peeked.kind,
                Ok(TokenKind::CloseBrace | TokenKind::CloseList)
            )
        }) {
            return Ok(List {
                values,
                closing_comments: prefix_comments,
            });
        }

        let mut value = parse_commented_value(tokens)?;
        {
            let mut prefix_comments = prefix_comments;
            prefix_comments.append(&mut value.prefix_comments);
            value.prefix_comments = prefix_comments;
        }

        values.push(value);

        // TODO: consume optional comma, with optional suffix comment
    }
}

/// Parse the inside of an object, without consuming either the opening or closing brackets.
fn parse_object_contents<'s>(tokens: &mut PeekableIter<'s>) -> Result<Object<'s>> {
    let mut key_values = vec![];

    loop {
        let prefix_comments = parse_comments(tokens);

        if tokens.peek().is_none_or(|peeked| {
            matches!(
                peeked.kind,
                Ok(TokenKind::CloseBrace | TokenKind::CloseList)
            )
        }) {
            return Ok(Object {
                key_values,
                closing_comments: prefix_comments,
            });
        }

        let Some(token) = tokens.next() else {
            return Ok(Object {
                key_values,
                closing_comments: prefix_comments,
            });
        };
        let token = token.ok()?;

        let key = match token.kind {
            TokenKind::Identifier => token,
            // TODO: quoted strings
            _ => {
                return Err(tokens.error_at(token.span, "Expected object key or '}'"));
            }
        };

        consume_token(tokens, TokenKind::Colon)?; // TODO: allow = too

        let mut value = parse_commented_value(tokens)?;
        {
            let mut prefix_comments = prefix_comments;
            prefix_comments.append(&mut value.prefix_comments);
            value.prefix_comments = prefix_comments;
        }

        key_values.push(KeyValue { key, value });

        // TODO: consume optional comma, with optional suffix comment
    }
}

/// Parse a value, including prefix and suffix comments.
fn parse_commented_value<'s>(tokens: &mut PeekableIter<'s>) -> Result<CommentedValue<'s>> {
    let start_span = tokens.span_of_next();
    let prefix_comments = parse_comments(tokens);

    let Some(result) = tokens.next() else {
        return Err(tokens.error_at(
            tokens.span_of_previous(),
            "Unexpected end of input: expected a value",
        ));
    };

    let token = result.ok()?;

    let value = match token.kind {
        TokenKind::OpenList => {
            let list = parse_list_contents(tokens)?;
            consume_token(tokens, TokenKind::CloseBrace)?;
            Value::List(list)
        }
        TokenKind::OpenBrace => {
            let object = parse_object_contents(tokens)?;
            consume_token(tokens, TokenKind::CloseBrace)?;
            Value::Object(object)
        }
        TokenKind::Identifier => Value::Identifier(token.slice),
        TokenKind::Number => Value::Number(token.slice),
        TokenKind::DoubleQuotedString | TokenKind::SingleQuotedString => Value::String(token.slice),
        _ => {
            return Err(tokens.error_at(
                token.span,
                "Expected a value, like an object, list, number, or string",
            ));
        }
    };

    let suffix_comment = parse_suffix_comment(tokens)?;

    Ok(CommentedValue {
        span: start_span | tokens.span_of_previous(),
        prefix_comments,
        value,
        suffix_comment,
    })
}

fn consume_token(tokens: &mut PeekableIter<'_>, expected_token: TokenKind) -> Result {
    if let Some(token) = tokens.next() {
        let token = token.ok()?;
        if token.kind == expected_token {
            Ok(())
        } else {
            Err(tokens.error_at(
                token.span,
                format!("Expected '{expected_token}' but found '{}'", token.kind),
            ))
        }
    } else {
        Err(tokens.error_at(
            tokens.span_of_previous(),
            format!("Expected '{expected_token:?}' but reached end of input"),
        ))
    }
}

fn parse_suffix_comment<'s>(tokens: &mut PeekableIter<'s>) -> Result<Option<&'s str>> {
    let previous_token_span = tokens.span_of_previous();
    let Some(token) = tokens.peek() else {
        return Ok(None);
    };
    if !matches!(token.kind, Ok(TokenKind::Comment)) {
        return Ok(None);
    }
    let comment_span = token.span;

    if tokens.source[previous_token_span.end..comment_span.start].contains('\n') {
        // The comment is not on the same line
        Ok(None)
    } else {
        // The comment is one the same line as the previous token (i.e. the value),
        // so this is a proper suffix comment.

        if let Some(token) = tokens.next() {
            let token = token.ok()?;
            debug_assert_eq!(
                token.kind,
                TokenKind::Comment,
                "Bug in parse_suffix_comment"
            );
            Ok(Some(token.slice))
        } else {
            Ok(None) // shouldn't be possible
        }
    }
}

fn parse_comments<'s>(tokens: &mut PeekableIter<'s>) -> Vec<&'s str> {
    let mut comments = vec![];
    while let Some(token) = tokens.peek() {
        if matches!(token.kind, Ok(TokenKind::Comment)) {
            comments.push(token.slice);
            tokens.next(); // Consume the comment token
        } else {
            break; // No more comments
        }
    }
    comments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_object() {
        let input = r#"
        // Prefix comment A.
        // Prefix comment B.
        key1: null

        // Prefix comment C.
        key2:
        // Prefix comment D.
        true // Suffix comment

        // Closing comment 1.
        // Closing comment 2.
        "#;

        let value = parse_top_str(input).unwrap();

        let CommentedValue {
            span: _,
            prefix_comments,
            value,
            suffix_comment,
        } = value;

        assert!(prefix_comments.is_empty());
        assert_eq!(suffix_comment, None);

        if let Value::Object(Object {
            key_values,
            closing_comments,
        }) = value
        {
            assert_eq!(key_values.len(), 2);

            {
                let KeyValue {
                    key,
                    value:
                        CommentedValue {
                            span: _,
                            prefix_comments,
                            value,
                            suffix_comment,
                        },
                } = &key_values[0];

                assert_eq!(key.slice, "key1");
                assert_eq!(
                    prefix_comments,
                    &["// Prefix comment A.", "// Prefix comment B."]
                );
                assert!(
                    matches!(value, &Value::Identifier("null")),
                    "Unexpected value: {value:?}"
                );
                assert_eq!(suffix_comment.as_deref(), None);
            }

            {
                let KeyValue {
                    key,
                    value:
                        CommentedValue {
                            span: _,
                            prefix_comments,
                            value,
                            suffix_comment,
                        },
                } = &key_values[1];

                assert_eq!(key.slice, "key2");
                assert_eq!(
                    prefix_comments,
                    &["// Prefix comment C.", "// Prefix comment D."]
                );
                assert!(
                    matches!(value, &Value::Identifier("true")),
                    "Unexpected value: {value:?}",
                );
                assert_eq!(suffix_comment.as_deref(), Some("// Suffix comment"));
            }

            assert_eq!(
                closing_comments,
                vec!["// Closing comment 1.", "// Closing comment 2."]
            );
        } else {
            panic!("Expected an object value, got {value:?}");
        }
    }
}
