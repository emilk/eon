//! Convert tokens into a token tree using a recursive descent parser.

use crate::{
    error::{Error, Result},
    span::Span,
    token_kind::TokenKind,
    token_tree::{TokenKeyValue, TokenList, TokenMap, TokenTree, TokenValue, TokenVariant},
};

/// Protect against stack overflow in our recursive descent parser.
const MAX_RECURSION_DEPTH: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct PlacedToken<'s> {
    /// The span of the token in the input string.
    pub span: Span,

    /// The token value
    pub slice: &'s str,

    /// The token type
    pub kind: TokenKind,
}

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
            Some(PlacedTokenResult {
                span,
                slice,
                kind: Err(Error::new_at(
                    self.iter.source(),
                    span,
                    format!("Invalid token: '{slice}'"),
                )),
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

    pub fn error_at(&self, span: Span, message: impl Into<String>) -> Error {
        Error::new_at(self.source, span, message)
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

    pub fn end_span(&self) -> Span {
        Span {
            start: self.source.len(),
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

impl<'s> TokenTree<'s> {
    /// Parse a full Eon file.
    pub fn parse_str(source: &'s str) -> Result<Self> {
        parse_top_str(source)
    }
}

/// Parse a full Eon file.
fn parse_top_str(eon_source: &str) -> Result<TokenTree<'_>> {
    // Usually an Eon file contains a bunch of `key: value` pairs, without any
    // surrounding braces, so we optimize for that case:
    let mut tokens_a = PeekableIter::new(eon_source);
    match parse_map_contents(&mut tokens_a, 0) {
        Ok(map) => {
            check_for_trailing_tokens(&mut tokens_a)?;
            let value = TokenTree {
                span: Some(Span {
                    start: 0,
                    end: eon_source.len(),
                }),
                prefix_comments: vec![],
                value: TokenValue::Map(map),
                suffix_comment: None,
            };
            Ok(value)
        }
        Err(err_a) => {
            // Maybe the use did wrap the file in {}, or maybe it is not an map?
            let mut tokens_b = PeekableIter::new(eon_source);

            match parse_list_contents(&mut tokens_b, 0) {
                Ok(list) => {
                    check_for_trailing_tokens(&mut tokens_b)?;

                    let TokenList {
                        values,
                        closing_comments,
                    } = list;

                    if values.len() == 1 {
                        // A file containing a single value, e.g. `42` or `{…}`,
                        Ok(values.into_iter().next().expect("Can't fail"))
                    } else {
                        // A file containing many values, e.g. `1, 2, 3` or `{…}, {…}`,
                        Ok(TokenTree {
                            span: Some(Span {
                                start: 0,
                                end: eon_source.len(),
                            }),
                            prefix_comments: Default::default(),
                            value: TokenValue::List(TokenList {
                                values,
                                closing_comments,
                            }),
                            suffix_comment: Default::default(),
                        })
                    }
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
fn parse_list_contents<'s>(
    tokens: &mut PeekableIter<'s>,
    recurse_depth: usize,
) -> Result<TokenList<'s>> {
    let mut values = vec![];

    loop {
        let prefix_comments = parse_comments(tokens);

        if tokens.peek().is_none_or(|peeked| {
            matches!(
                peeked.kind,
                Ok(TokenKind::CloseBrace | TokenKind::CloseList | TokenKind::CloseParen)
            )
        }) {
            return Ok(TokenList {
                values,
                closing_comments: prefix_comments,
            });
        }

        let mut value = parse_token_tree(tokens, recurse_depth + 1)?;

        {
            let mut prefix_comments = prefix_comments;
            prefix_comments.append(&mut value.prefix_comments);
            value.prefix_comments = prefix_comments;
        }

        if tokens
            .peek()
            .is_some_and(|peeked| matches!(peeked.kind, Ok(TokenKind::Comma)))
        {
            // Consume optional comma
            tokens.next();
            value.suffix_comment = parse_suffix_comment(tokens)?;
        }

        values.push(value);
    }
}

/// Parse the inside of an map, without consuming either the opening or closing brackets.
fn parse_map_contents<'s>(
    tokens: &mut PeekableIter<'s>,
    recurse_depth: usize,
) -> Result<TokenMap<'s>> {
    let mut key_values = vec![];

    loop {
        let prefix_comments = parse_comments(tokens);

        if tokens.peek().is_none_or(|peeked| {
            matches!(
                peeked.kind,
                Ok(TokenKind::CloseBrace | TokenKind::CloseList | TokenKind::CloseParen)
            )
        }) {
            return Ok(TokenMap {
                key_values,
                closing_comments: prefix_comments,
            });
        }

        let mut key = parse_token_tree(tokens, recurse_depth + 1)?;
        debug_assert!(
            key.prefix_comments.is_empty(),
            "We should have already consumed these"
        );
        key.prefix_comments = prefix_comments;

        consume_token(tokens, TokenKind::Colon)?;

        let mut value = parse_token_tree(tokens, recurse_depth + 1)?;

        if tokens
            .peek()
            .is_some_and(|peeked| matches!(peeked.kind, Ok(TokenKind::Comma)))
        {
            // Consume optional comma
            tokens.next();
            value.suffix_comment = parse_suffix_comment(tokens)?;
        }

        key_values.push(TokenKeyValue { key, value });
    }
}

/// Parse a value, including prefix and suffix comments.
fn parse_token_tree<'s>(
    tokens: &mut PeekableIter<'s>,
    recurse_depth: usize,
) -> Result<TokenTree<'s>> {
    if recurse_depth >= MAX_RECURSION_DEPTH {
        return Err(tokens.error_at(
            tokens.span_of_previous(),
            "Maximum recursion depth exceeded while parsing document",
        ));
    }

    let prefix_comments = parse_comments(tokens);

    let Some(result) = tokens.next() else {
        return Err(tokens.error_at(
            tokens.end_span(),
            "Unexpected end of input: expected a value",
        ));
    };

    let token = result.ok()?;

    let start_span = tokens.span_of_next();

    let value = match token.kind {
        TokenKind::OpenList => {
            let list = parse_list_contents(tokens, recurse_depth + 1)?;
            consume_token(tokens, TokenKind::CloseList)?;
            TokenValue::List(list)
        }
        TokenKind::OpenBrace => {
            let map = parse_map_contents(tokens, recurse_depth + 1)?;
            consume_token(tokens, TokenKind::CloseBrace)?;
            TokenValue::Map(map)
        }
        TokenKind::Identifier => TokenValue::Identifier(token.slice.into()),
        TokenKind::Number => TokenValue::Number(token.slice.into()),
        TokenKind::DoubleQuotedString
        | TokenKind::SingleQuotedString
        | TokenKind::MultilineBasicString
        | TokenKind::MultilineLiteralString => {
            // This could be a free-floating string
            // or the opening of a variant, like `"Rgb"(…)`:

            if tokens
                .peek()
                .is_some_and(|peeked| matches!(peeked.kind, Ok(TokenKind::OpenParen)))
            {
                tokens.next(); // Consume the open parenthesis

                let TokenList {
                    values,
                    closing_comments,
                } = parse_list_contents(tokens, recurse_depth + 1)?;

                consume_token(tokens, TokenKind::CloseParen)?;

                TokenValue::Variant(TokenVariant {
                    name_span: Some(token.span),
                    quoted_name: token.slice.into(),
                    values,
                    closing_comments,
                })
            } else {
                // Just a string, not a variant
                TokenValue::QuotedString(token.slice.into())
            }
        }
        TokenKind::Comment => unreachable!("We should have already consumed comments"),
        TokenKind::CloseList => Err(tokens.error_at(token.span, "Unbalanced brackets"))?,
        TokenKind::CloseBrace => Err(tokens.error_at(token.span, "Unbalanced braces"))?,
        TokenKind::CloseParen => Err(tokens.error_at(token.span, "Unbalanced parentheses"))?,
        TokenKind::OpenParen => {
            Err(tokens.error_at(token.span, "Parentheses must be proceeded by a string"))?
        }
        TokenKind::Colon | TokenKind::Comma => {
            return Err(tokens.error_at(
                token.span,
                "Expected a value, like a map, list, number, or string",
            ));
        }
    };

    let span = start_span | tokens.span_of_previous();

    let suffix_comment = parse_suffix_comment(tokens)?;

    Ok(TokenTree {
        span: Some(span),
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
                format!("Expected {expected_token} but found {}", token.kind),
            ))
        }
    } else {
        Err(tokens.error_at(
            tokens.span_of_previous(),
            format!("Expected {expected_token} but reached end of input"),
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
    fn test_parse_map() {
        let input = r#"
        // Prefix comment A.
        // Prefix comment B.
        key1: 42

        // Prefix comment C.
        key2:
        // Prefix comment D.
        "string" // Suffix comment

        // Closing comment 1.
        // Closing comment 2.
        "#;

        let value = parse_top_str(input).unwrap();

        let TokenTree {
            span: _,
            prefix_comments,
            value,
            suffix_comment,
        } = value;

        assert!(prefix_comments.is_empty());
        assert_eq!(suffix_comment, None);

        if let TokenValue::Map(TokenMap {
            key_values,
            closing_comments,
        }) = value
        {
            assert_eq!(key_values.len(), 2);

            {
                let TokenKeyValue { key, value } = &key_values[0];

                if let TokenValue::Identifier(key) = &key.value {
                    assert_eq!(key, "key1");
                } else {
                    panic!("Expected an identifier for key1, got {key:?}");
                }
                assert_eq!(
                    key.prefix_comments,
                    &["// Prefix comment A.", "// Prefix comment B."]
                );
                if let TokenValue::Number(value) = &value.value {
                    assert_eq!(value, "42");
                } else {
                    panic!("Expected a number for key1, got {key:?}");
                }
                assert_eq!(value.suffix_comment, None);
            }

            {
                let TokenKeyValue { key, value } = &key_values[1];

                if let TokenValue::Identifier(key) = &key.value {
                    assert_eq!(key, "key2");
                } else {
                    panic!("Expected an identifier for key1, got {key:?}");
                }
                assert_eq!(key.prefix_comments, &["// Prefix comment C."]);
                if let TokenValue::QuotedString(value) = &value.value {
                    assert_eq!(value, r#""string""#);
                } else {
                    panic!("Expected a String for key2, got {key:?}");
                }
                assert_eq!(value.prefix_comments, ["// Prefix comment D."]);
                assert_eq!(value.suffix_comment, Some("// Suffix comment"));
            }

            assert_eq!(
                closing_comments,
                vec!["// Closing comment 1.", "// Closing comment 2."]
            );
        } else {
            panic!("Expected a map value, got {value:?}");
        }
    }
}
