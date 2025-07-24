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

    /// `:`
    #[token(":")]
    Colon,

    /// `=`
    #[token("=")]
    Equals,

    /// `,`
    #[token(",")]
    Comma,

    /// Can be an object key, or "false", "true", "null"
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    /// `"this"`
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleQuotedString,

    /// `'this'`
    #[regex(r#"'([^'\\]|\\.)*'"#)]
    SingleQuotedString,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comment => write!(f, "// comment"),
            Self::OpenList => write!(f, "["),
            Self::CloseList => write!(f, "]"),
            Self::OpenBrace => write!(f, "{{"),
            Self::CloseBrace => write!(f, "}}"),
            Self::Colon => write!(f, ":"),
            Self::Equals => write!(f, "="),
            Self::Comma => write!(f, ","),
            Self::Identifier => write!(f, "identifier"),
            Self::DoubleQuotedString => write!(f, r#""double quoted" string"#),
            Self::SingleQuotedString => write!(f, r"'single quoted' string"),
        }
    }
}

#[test]
fn test_parse() {
    let input = r#"
    // Comment
    single: 'single'
    "double"
    [ { },]
    "#;

    let expect = [
        (TokenKind::Comment, "// Comment"),
        (TokenKind::Identifier, "single"),
        (TokenKind::Colon, ":"),
        (TokenKind::SingleQuotedString, "'single'"),
        (TokenKind::DoubleQuotedString, "\"double\""),
        (TokenKind::OpenList, "["),
        (TokenKind::OpenBrace, "{"),
        (TokenKind::CloseBrace, "}"),
        (TokenKind::Comma, ","),
        (TokenKind::CloseList, "]"),
    ];
    let mut lexer = TokenKind::lexer(input);

    for (expected_token, expected_text) in &expect {
        let token = lexer
            .next()
            .expect("Expected something")
            .expect("Expected a valid token");
        assert_eq!(token, *expected_token, "Token mismatch");
        assert_eq!(lexer.slice(), *expected_text, "Text mismatch");
    }
    assert!(lexer.next().is_none(), "Expected no more tokens");
}
