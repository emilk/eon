use logos::Logos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\n\f]*")] // Ignore this regex pattern between tokens
pub enum Token {
    /// `// Some comment`
    #[regex("//[^\n]*")]
    Comment,

    /// `{`
    #[token("{")]
    OpenBrace,

    /// `}`
    #[token("}")]
    CloseBrace,

    /// `[`
    #[token("[")]
    OpenList,

    /// `]`
    #[token("]")]
    CloseList,

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

#[test]
fn test_parse() {
    let input = r#"
    // Comment
    single: 'single'
    "double"
    [ { },]
    "#;

    let expect = [
        (Token::Comment, "// Comment"),
        (Token::Identifier, "single"),
        (Token::Colon, ":"),
        (Token::SingleQuotedString, "'single'"),
        (Token::DoubleQuotedString, "\"double\""),
        (Token::OpenList, "["),
        (Token::OpenBrace, "{"),
        (Token::CloseBrace, "}"),
        (Token::Comma, ","),
        (Token::CloseList, "]"),
    ];
    let mut lexer = Token::lexer(input);

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
