use crate::{Error, Result, ast::*, span::Span, token::TokenType};

pub struct PlacedTokenResult<'s> {
    /// The span of the token in the input string.
    pub span: Span,

    /// The token value
    pub slice: &'s str,

    /// The token type
    pub token: Result<TokenType>,
}

impl<'s> PlacedTokenResult<'s> {
    /// Returns the inner token, or an error if the token is invalid.
    pub fn ok(self) -> Result<PlacedToken<'s>> {
        match self.token {
            Ok(token) => Ok(PlacedToken {
                span: self.span,
                slice: self.slice,
                token,
            }),
            Err(err) => Err(err),
        }
    }
}

pub struct PlacedTokenIter<'s> {
    iter: logos::SpannedIter<'s, TokenType>,
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
        match result {
            Ok(token) => Some(PlacedTokenResult {
                span,
                slice,
                token: Ok(token),
            }),
            Err(()) => {
                let error = Error::build(ariadne::ReportKind::Error, span.clone())
                    .with_message("Invalid token")
                    .finish();
                Some(PlacedTokenResult {
                    span,
                    slice,
                    token: Err(error),
                })
            }
        }
    }
}

pub struct PeekableIter<'s> {
    source: &'s str,
    iter: PlacedTokenIter<'s>,

    /// Remember a peeked value, even if it was None.
    peeked: Option<Option<PlacedTokenResult<'s>>>,

    last_span: Span,
}

impl<'s> PeekableIter<'s> {
    pub fn new(source: &'s str) -> Self {
        use logos::Logos as _;
        PeekableIter {
            source,
            iter: PlacedTokenIter {
                iter: TokenType::lexer(source).spanned(),
            },
            peeked: None,
            last_span: Span { start: 0, end: 0 },
        }
    }

    pub fn peek(&mut self) -> Option<&PlacedTokenResult<'s>> {
        let iter = &mut self.iter;
        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    pub fn next(&mut self) -> Option<PlacedTokenResult<'s>> {
        let next = match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next(),
        };
        if let Some(next) = &next {
            self.last_span = next.span;
        }
        next
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
}

/// Parse a full Con file.
pub fn parse_top_str(source: &str) -> Result<CommentedValue<'_>> {
    // Usually a Con file contains a bunch of `key: value` pairs, without any
    // surrounding braces, so we optimize for that case:
    let mut tokens = PeekableIter::new(source);
    match parse_object_contents(&mut tokens) {
        Ok(node) => {
            check_for_trailing_tokens(&mut tokens)?;
            Ok(node)
        }
        Err(err) => {
            // Maybe the use did wrap the file in {}, or maybe it is not an object?
            let mut tokens = PeekableIter::new(source);
            if let Ok(value) = parse_commented_value(&mut tokens) {
                check_for_trailing_tokens(&mut tokens)?;
                Ok(value)
            } else {
                // TODO: figure out which error to return here.
                Err(err)
            }
        }
    }
}

fn check_for_trailing_tokens(tokens: &mut PeekableIter<'_>) -> Result {
    if let Some(token) = tokens.next() {
        Err(Error::build(ariadne::ReportKind::Error, token.span)
            .with_message("Trailing tokens found after parsing")
            .finish())
    } else {
        Ok(())
    }
}

fn parse_object_contents<'s>(tokens: &mut PeekableIter<'s>) -> Result<CommentedValue<'s>> {
    let start_span = tokens.span_of_next();
    let mut key_values = vec![];

    loop {
        let prefix_comments = parse_comments(tokens)?;
        let Some(token) = tokens.peek() else {
            // No more tokens
            return Ok(CommentedValue {
                span: start_span | tokens.span_of_previous(),
                prefix_comments: vec![],
                value: Value::Object(Object {
                    key_values,
                    closing_comments: prefix_comments,
                }),
                suffix_comment: None,
            });
        };

        if matches!(
            token.token,
            Ok(TokenType::CloseBrace | TokenType::CloseList)
        ) {
            // End of the object
            return Ok(CommentedValue {
                span: start_span | tokens.span_of_previous(),
                prefix_comments: vec![],
                value: Value::Object(Object {
                    key_values,
                    closing_comments: prefix_comments,
                }),
                suffix_comment: None,
            });
        }

        let token = tokens.next().unwrap().ok()?;

        let key = match token.token {
            TokenType::Identifier => token,
            // TODO: quoted strings
            _ => {
                return Err(Error::build(ariadne::ReportKind::Error, token.span)
                    .with_message("Expected object key or '}'")
                    .finish());
            }
        };

        match tokens.next() {
            Some(next_token) => {
                if !matches!(next_token.token, Ok(TokenType::Colon)) {
                    return Err(Error::build(ariadne::ReportKind::Error, next_token.span)
                        .with_message("Expected ':' after object key")
                        .finish());
                }
            }
            None => {
                return Err(Error::build(ariadne::ReportKind::Error, token.span)
                    .with_message("Unexpected end of input: expected ':'")
                    .finish());
            }
        }

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
    let prefix_comments = parse_comments(tokens)?;

    let Some(result) = tokens.next() else {
        return Err(
            Error::build(ariadne::ReportKind::Error, Span { start: 0, end: 0 }) // TODO: correct span
                .with_message("Unexpected end of input: expected a value")
                .finish(),
        );
    };

    let token = result.ok()?;

    let value = match token.token {
        TokenType::OpenBrace => unimplemented!(),
        TokenType::OpenList => unimplemented!(),
        TokenType::Identifier => Value::Identifier(token.slice),
        TokenType::DoubleQuotedString => unimplemented!(),
        TokenType::SingleQuotedString => unimplemented!(),
        _ => {
            return Err(Error::build(ariadne::ReportKind::Error, token.span)
                .with_message("Expected a value, like an object, list, number, or string")
                .finish());
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

fn parse_suffix_comment<'s>(tokens: &mut PeekableIter<'s>) -> Result<Option<&'s str>> {
    let previous_token_span = tokens.span_of_previous();
    let Some(token) = tokens.peek() else {
        return Ok(None);
    };
    if !matches!(token.token, Ok(TokenType::Comment)) {
        return Ok(None);
    }
    let comment_span = token.span;

    if tokens.source[previous_token_span.end..comment_span.start].contains('\n') {
        // The comment is not on the same line
        return Ok(None);
    }

    // The comment is one the same line as the previous token (i.e. the value),
    // so this is a proper suffix comment.

    let token = tokens.next().unwrap().ok()?;
    Ok(Some(token.slice))
}

fn parse_comments<'s>(tokens: &mut PeekableIter<'s>) -> Result<Vec<&'s str>> {
    let mut comments = vec![];
    while let Some(token) = tokens.peek() {
        if matches!(token.token, Ok(TokenType::Comment)) {
            comments.push(token.slice);
            tokens.next(); // Consume the comment token
        } else {
            break; // No more comments
        }
    }
    Ok(comments)
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

        let value = parse_top_str(input).expect("Failed to parse object");

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
