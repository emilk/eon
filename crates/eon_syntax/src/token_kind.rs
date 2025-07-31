use logos::Logos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\n\f]*")] // Ignore this regex pattern between tokens
pub enum TokenKind {
    /// `// Some comment`
    #[regex("//[^\n]*")]
    Comment,

    /// `[`
    #[token("[")]
    OpenList,

    /// `]`
    #[token("]")]
    CloseList,

    /// `{`
    #[token("{")]
    OpenBrace,

    /// `}`
    #[token("}")]
    CloseBrace,

    /// `(`
    #[token("(")]
    OpenParen,

    /// `)`
    #[token(")")]
    CloseParen,

    /// `:`
    #[token(":")]
    Colon,

    /// `,`
    #[token(",")]
    Comma,

    /// Can be an map key, or "false", "true", "null"
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    /// Anything that starts with a sign (+/-), a digit (0-9), or a period (decimal separator).
    #[regex("[+\\-0-9\\.][0-9a-zA-Z\\.+\\-_]*")]
    Number,

    /// `"this"`
    ///
    /// Processes escaped characters like `\"`, `\\`, `\n`, etc.
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleQuotedString,

    /// Raw string `'like "this"'`
    ///
    /// Can contain any character except for single quotes.
    /// Does not process escape sequences.
    #[regex(r"'([^'])*'")]
    SingleQuotedString,

    // Multiline basic string (triple double quotes) - processes escape sequences
    #[regex(r#""""([^"]|"[^"]|""[^"])*""""#)]
    MultilineBasicString,

    // Multiline literal string (triple single quotes) - no escape processing
    #[regex(r"'''([^']|'[^']|''[^'])*'''")]
    MultilineLiteralString,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comment => write!(f, "// comment"),
            Self::OpenList => write!(f, "open bracket '['"),
            Self::CloseList => write!(f, "close bracket ']'"),
            Self::OpenBrace => write!(f, "open brace '{{'"),
            Self::CloseBrace => write!(f, "close brace '}}'"),
            Self::OpenParen => write!(f, "open parenthesis '('"),
            Self::CloseParen => write!(f, "close parenthesis ')'"),
            Self::Colon => write!(f, "colon ':'"),
            Self::Comma => write!(f, "comma ','"),
            Self::Identifier => write!(f, "identifier"),
            Self::Number => write!(f, "number"),
            Self::DoubleQuotedString => write!(f, r#""basic string""#),
            Self::SingleQuotedString => write!(f, r"'literal string'"),
            Self::MultilineBasicString => write!(f, r#"""multiline basic string"""#),
            Self::MultilineLiteralString => write!(f, r#"'''multiline literal string'''"#),
        }
    }
}

#[test]
fn test_parse_tokens() {
    let input = r#"
    // Comment
    key: value
    [ { },]
    42
    123_456
    +inf
    +1.e3-42
    0xdeadbeef
    "basic \n \"string\""
    'single quoted string'
    """
    multiline
    basic \
    string"""
    '''
multiline
literal
string'''
    "#;

    let expect = [
        (TokenKind::Comment, "// Comment"),
        (TokenKind::Identifier, "key"),
        (TokenKind::Colon, ":"),
        (TokenKind::Identifier, "value"),
        (TokenKind::OpenList, "["),
        (TokenKind::OpenBrace, "{"),
        (TokenKind::CloseBrace, "}"),
        (TokenKind::Comma, ","),
        (TokenKind::CloseList, "]"),
        (TokenKind::Number, "42"),
        (TokenKind::Number, "123_456"),
        (TokenKind::Number, "+inf"),
        (TokenKind::Number, "+1.e3-42"),
        (TokenKind::Number, "0xdeadbeef"),
        (TokenKind::DoubleQuotedString, r#""basic \n \"string\"""#),
        (TokenKind::SingleQuotedString, "'single quoted string'"),
        (
            TokenKind::MultilineBasicString,
            "\"\"\"\n    multiline\n    basic \\\n    string\"\"\"",
        ),
        (
            TokenKind::MultilineLiteralString,
            "'''\nmultiline\nliteral\nstring'''",
        ),
    ];
    let mut lexer = TokenKind::lexer(input);

    for (expected_token, expected_text) in &expect {
        let actual_token = lexer
            .next()
            .expect("Expected something")
            .unwrap_or_else(|()| panic!("Not a valid token: {:?}", lexer.slice()));
        let actual_text = lexer.slice();
        assert!(
            actual_token == *expected_token && actual_text == *expected_text,
            "Token mismatch. Expected {expected_token} ({expected_text:?}), got {actual_token} ({actual_text:?})"
        );
    }
    assert!(lexer.next().is_none(), "Expected no more tokens");
}
