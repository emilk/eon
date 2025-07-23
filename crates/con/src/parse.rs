use logos::Logos;

use crate::token::Token;

#[derive(Clone, Copy, Debug)]
pub struct Span {
    start: usize,
    end: usize,
}

impl ariadne::Span for Span {
    type SourceId = ();

    fn source(&self) -> &Self::SourceId {
        &()
    }
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
}

/// Represent an error during parsing
pub type Error = ariadne::Report<'static, Span>;

/// A type alias for a result that uses the [`Error`] type defined above.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// Captures the comments related to a Config value.
#[derive(Default)]
struct Comments<'s> {
    /// Comments on preceeding lines.
    prefix: Vec<&'s str>,

    /// After the value, on the same line.  `value // Like this`.
    suffix: Option<&'s str>,

    /// Before the closing } or ]
    pre_end_brace: Vec<&'s str>,
}

struct CommentedValue<'s> {
    /// Comments on preceeding lines.
    ///
    /// ```
    /// // Like this
    /// // and this.
    /// 42
    /// ```
    prefix_comments: Vec<&'s str>,

    /// The actual value.
    value: Value<'s>,

    /// Comment after the value on the same line.
    ///
    /// `value // Like this`.
    suffix_comment: Option<&'s str>,
}

struct KeyValue<'s> {
    /// The key of the key-value pair.
    ///
    /// Includes quotes if the key is a quoted string.
    key: PlacedToken<'s>,

    /// The value of the key-value pair.
    ///
    /// Also contains any comments before the key, before the value, and after the value.
    value: CommentedValue<'s>,
}

enum Value<'s> {
    Null,

    Bool(bool),

    /// A number, like `42`, `3.14`, `-1.0`, `0x53`, `+NaN` etc.
    Number(&'s str),

    /// Includes the actual quotes of the string, both opening and closing.
    String(&'s str),

    /// An object, like `{ key: value }`.
    ///
    /// If the key is a quoted string, the key will include the quotes.
    Object {
        key_values: Vec<KeyValue<'s>>,

        /// Any comments after the last `key: value` pair, before the closing `}`.
        closing_comments: Vec<&'s str>,
    },

    /// A list, like `[ value, value ]`.
    List {
        values: Vec<CommentedValue<'s>>,

        /// Any comments after the last value, before the closing `]`.
        closing_comments: Vec<&'s str>,
    },
}

pub struct PlacedTokenResult<'s> {
    /// The span of the token in the input string.
    pub span: Span,

    /// The token value
    pub slice: &'s str,

    /// The token type
    pub token: Result<Token>,
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

#[derive(Clone, Copy, Debug)]
pub struct PlacedToken<'s> {
    /// The span of the token in the input string.
    pub span: Span,

    /// The token value
    pub slice: &'s str,

    /// The token type
    pub token: Token,
}

pub struct PlacedTokenIter<'s> {
    iter: logos::SpannedIter<'s, Token>,
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

type PeekableIter<'s> = std::iter::Peekable<PlacedTokenIter<'s>>;

fn new_peekable_iter<'s>(input: &'s str) -> PeekableIter<'s> {
    PlacedTokenIter {
        iter: Token::lexer(input).spanned(),
    }
    .peekable()
}

/// Parse a full Con file.
pub fn parse_top_str(input: &str) -> Result<CommentedValue<'_>> {
    // Usually a Con file contains a bunch of `key: value` pairs, without any
    // surrounding braces, so we optimize for that case:
    let mut tokens = new_peekable_iter(input);
    match parse_object_contents(&mut tokens) {
        Ok(node) => {
            check_for_trailing_tokens(&mut tokens)?;
            Ok(node)
        }
        Err(err) => {
            // Maybe the use did wrap the file in {}, or maybe it is not an object?
            let mut tokens = new_peekable_iter(input);
            if let Ok(value) = parse_commentedvalue(&mut tokens) {
                check_for_trailing_tokens(&mut tokens)?;
                Ok(value)
            } else {
                // TODO: figure out which error to return here.
                Err(err)
            }
        }
    }
}

fn check_for_trailing_tokens<'s>(tokens: &mut PeekableIter<'s>) -> Result {
    if let Some(token) = tokens.next() {
        Err(Error::build(ariadne::ReportKind::Error, token.span)
            .with_message("Trailing tokens found after parsing")
            .finish())
    } else {
        Ok(())
    }
}

fn parse_object_contents<'s>(tokens: &mut PeekableIter<'s>) -> Result<CommentedValue<'s>> {
    let mut key_values = vec![];

    loop {
        let prefix_comments = parse_comments(tokens)?;
        let Some(token) = tokens.peek() else {
            // No more tokens
            return Ok(CommentedValue {
                prefix_comments: vec![],
                value: Value::Object {
                    key_values,
                    closing_comments: prefix_comments,
                },
                suffix_comment: None,
            });
        };
        let token = tokens.next().unwrap().ok()?;

        let key = match token.token {
            Token::CloseBrace | Token::CloseList => {
                // End of the object
                return Ok(CommentedValue {
                    prefix_comments: vec![],
                    value: Value::Object {
                        key_values,
                        closing_comments: prefix_comments,
                    },
                    suffix_comment: None,
                });
            }
            Token::Identifier => token,
            // TODO: quoted strings
            _ => {
                return Err(Error::build(ariadne::ReportKind::Error, token.span)
                    .with_message("Expected object key or '}'")
                    .finish());
            }
        };

        match tokens.next() {
            Some(next_token) => {
                if !matches!(next_token.token, Ok(Token::Colon)) {
                    return Err(Error::build(ariadne::ReportKind::Error, next_token.span)
                        .with_message("Expected ':' after object key")
                        .finish());
                }
            }
            None => {
                return Err(Error::build(ariadne::ReportKind::Error, token.span)
                    .with_message("Unexpected end of input, expected ':'")
                    .finish());
            }
        }

        let mut value = parse_commentedvalue(tokens)?;
        let suffix_comment = parse_suffix_comment(tokens)?; // TODO: should already have been consumed by parse_value
        {
            let mut prefix_comments = prefix_comments;
            prefix_comments.append(&mut value.prefix_comments);
            value.prefix_comments = prefix_comments;
        }

        key_values.push(KeyValue { key, value });
    }
}

/// Parse a value, including prefix and suffix comments.
fn parse_commentedvalue<'s>(
    tokens: &mut std::iter::Peekable<PlacedTokenIter<'s>>,
) -> Result<CommentedValue<'s>> {
    todo!()
}

fn parse_suffix_comment<'s>(
    tokens: &mut std::iter::Peekable<PlacedTokenIter<'s>>,
) -> Result<Option<&'s str>> {
    todo!()
}

fn parse_comments<'s>(tokens: &mut PeekableIter<'s>) -> Result<Vec<&'s str>> {
    let mut comments = vec![];
    while let Some(token) = tokens.peek() {
        if matches!(token.token, Ok(Token::Comment)) {
            comments.push(token.slice);
            tokens.next(); // Consume the comment token
        } else {
            break; // No more comments
        }
    }
    Ok(comments)
}
