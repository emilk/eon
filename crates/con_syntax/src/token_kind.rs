use logos::Logos;

/// Returns `true` if the string does NOT match `[a-zA-Z_][a-zA-Z0-9_]*`
pub fn needs_quotes(string: &str) -> bool {
    let mut chars = string.chars();

    if chars
        .next()
        .is_none_or(|c| !c.is_ascii_alphabetic() && c != '_')
    {
        return true;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return true;
        }
    }

    false
}
#[test]
fn test_needs_quotes() {
    assert!(needs_quotes(""));
    assert!(!needs_quotes("a"));
    assert!(!needs_quotes("a1"));
    assert!(!needs_quotes("a_b"));
    assert!(!needs_quotes("a_b1"));
    assert!(needs_quotes("1a"));
    assert!(!needs_quotes("_1a"));
    assert!(needs_quotes("a-b"));
    assert!(needs_quotes("a b"));
}

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
    #[regex("[+\\-0-9\\.][0-9a-zA-Z\\.+\\-]*")]
    Number,

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
            Self::DoubleQuotedString => write!(f, r#""double quoted" string"#),
            Self::SingleQuotedString => write!(f, r"'single quoted' string"),
        }
    }
}

#[test]
fn test_parse_tokens() {
    let input = r#"
    // Comment
    single: 'single'
    "double"
    [ { },]
    42
    +inf
    +1.e3-42
    0xdeadbeef
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
        (TokenKind::Number, "42"),
        (TokenKind::Number, "+inf"),
        (TokenKind::Number, "+1.e3-42"),
        (TokenKind::Number, "0xdeadbeef"),
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
