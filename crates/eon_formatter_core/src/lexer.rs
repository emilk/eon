use alloc::vec::Vec;

use crate::{
    Error, ErrorKind, Result, Span,
    token::{Item, StringKind, Token, TokenKind, TokenStream, Trivia, TriviaKind},
};

/// Lex Eon source into a borrowed stream of tokens and trivia for formatting.
pub fn lex(source: &str) -> Result<TokenStream<'_>> {
    Lexer::new(source).lex_document()
}

struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    fn lex_document(mut self) -> Result<TokenStream<'a>> {
        let mut items = Vec::new();

        while !self.is_eof() {
            self.lex_trivia(&mut items)?;
            if self.is_eof() {
                break;
            }
            items.push(Item::Token(self.lex_token()?));
        }

        Ok(TokenStream::new(items))
    }

    fn lex_trivia(&mut self, items: &mut Vec<Item<'a>>) -> Result {
        loop {
            match self.peek_byte() {
                Some(b' ' | b'\t' | b'\n' | b'\r' | 0x0C) => {
                    items.push(Item::Trivia(self.lex_whitespace()));
                }
                Some(b'/') if self.starts_with(b"//") => {
                    items.push(Item::Trivia(self.lex_comment()?));
                }
                _ => return Ok(()),
            }
        }
    }

    fn lex_whitespace(&mut self) -> Trivia<'a> {
        let start = self.pos;
        while matches!(self.peek_byte(), Some(b' ' | b'\t' | b'\n' | b'\r' | 0x0C)) {
            self.pos += 1;
        }
        let span = Span::new(start, self.pos);
        let raw = &self.source[start..self.pos];
        Trivia {
            span,
            kind: TriviaKind::Whitespace,
            raw,
            line_breaks: count_line_breaks(raw),
        }
    }

    fn lex_comment(&mut self) -> Result<Trivia<'a>> {
        let start = self.pos;
        self.pos += 2; // //

        while let Some(byte) = self.peek_byte() {
            if matches!(byte, b'\n' | b'\r') {
                break;
            }
            self.pos += 1;
        }

        let span = Span::new(start, self.pos);
        let raw = &self.source[start..self.pos];
        validate_source_slice(raw, span)?;

        Ok(Trivia {
            span,
            kind: TriviaKind::Comment,
            raw,
            line_breaks: 0,
        })
    }

    fn lex_token(&mut self) -> Result<Token<'a>> {
        let start = self.pos;
        let kind = match self.peek_byte() {
            Some(b'[') => {
                self.pos += 1;
                TokenKind::OpenList
            }
            Some(b']') => {
                self.pos += 1;
                TokenKind::CloseList
            }
            Some(b'{') => {
                self.pos += 1;
                TokenKind::OpenBrace
            }
            Some(b'}') => {
                self.pos += 1;
                TokenKind::CloseBrace
            }
            Some(b'(') => {
                self.pos += 1;
                TokenKind::OpenParen
            }
            Some(b')') => {
                self.pos += 1;
                TokenKind::CloseParen
            }
            Some(b':') => {
                self.pos += 1;
                TokenKind::Colon
            }
            Some(b',') => {
                self.pos += 1;
                TokenKind::Comma
            }
            Some(b'"' | b'\'') => self.lex_string_kind()?,
            Some(b'+' | b'-' | b'.' | b'0'..=b'9') => {
                self.lex_number();
                TokenKind::Number
            }
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
                self.lex_identifier();
                TokenKind::Identifier
            }
            Some(byte) => {
                return Err(Error::new(
                    Span::new(self.pos, self.pos + 1),
                    ErrorKind::UnexpectedByte(byte),
                ));
            }
            None => unreachable!("lex_token is never called at EOF"),
        };

        let span = Span::new(start, self.pos);
        Ok(Token {
            span,
            kind,
            raw: &self.source[start..self.pos],
        })
    }

    fn lex_identifier(&mut self) {
        self.pos += 1;
        while matches!(
            self.peek_byte(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        ) {
            self.pos += 1;
        }
    }

    fn lex_number(&mut self) {
        self.pos += 1;
        while matches!(
            self.peek_byte(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'+' | b'-' | b'.' | b'_')
        ) {
            self.pos += 1;
        }
    }

    fn lex_string_kind(&mut self) -> Result<TokenKind> {
        let start = self.pos;

        if self.starts_with(b"\"\"\"") {
            self.pos += 3;
            while !self.is_eof() {
                if self.starts_with(b"\"\"\"") {
                    self.pos += 3;
                    let span = Span::new(start, self.pos);
                    validate_source_slice(&self.source[start..self.pos], span)?;
                    return Ok(TokenKind::String(StringKind::MultilineBasic));
                }
                self.pos += 1;
            }
        } else if self.starts_with(b"'''") {
            self.pos += 3;
            while !self.is_eof() {
                if self.starts_with(b"'''") {
                    self.pos += 3;
                    let span = Span::new(start, self.pos);
                    validate_source_slice(&self.source[start..self.pos], span)?;
                    return Ok(TokenKind::String(StringKind::MultilineLiteral));
                }
                self.pos += 1;
            }
        } else if self.peek_byte() == Some(b'"') {
            self.pos += 1;
            while let Some(byte) = self.peek_byte() {
                match byte {
                    b'\\' => {
                        self.pos += 1;
                        if self.is_eof() {
                            break;
                        }
                        self.pos += 1;
                    }
                    b'"' => {
                        self.pos += 1;
                        let span = Span::new(start, self.pos);
                        validate_source_slice(&self.source[start..self.pos], span)?;
                        return Ok(TokenKind::String(StringKind::Basic));
                    }
                    b'\n' | b'\r' => break,
                    _ => self.pos += 1,
                }
            }
        } else if self.peek_byte() == Some(b'\'') {
            self.pos += 1;
            while let Some(byte) = self.peek_byte() {
                match byte {
                    b'\'' => {
                        self.pos += 1;
                        let span = Span::new(start, self.pos);
                        validate_source_slice(&self.source[start..self.pos], span)?;
                        return Ok(TokenKind::String(StringKind::Literal));
                    }
                    b'\n' | b'\r' => break,
                    _ => self.pos += 1,
                }
            }
        }

        Err(Error::new(
            Span::new(start, self.pos.min(self.source.len())),
            ErrorKind::UnterminatedString,
        ))
    }

    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.bytes[self.pos..].starts_with(prefix)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }
}

fn validate_source_slice(source: &str, span: Span) -> Result {
    if let Some((offset, chr)) = find_disallowed_invisible_unicode(source) {
        return Err(Error::new(
            Span::new(span.start + offset, span.start + offset + chr.len_utf8()),
            ErrorKind::DisallowedInvisibleUnicode(chr),
        ));
    }

    Ok(())
}

fn find_disallowed_invisible_unicode(source: &str) -> Option<(usize, char)> {
    source.char_indices().find(|&(_, chr)| {
        !matches!(chr, '"' | '\\' | '\'' | '\n' | '\r' | '\t')
            && chr.escape_debug().next() == Some('\\')
    })
}

fn count_line_breaks(raw: &str) -> usize {
    let bytes = raw.as_bytes();
    let mut count = 0;
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                count += 1;
                index += 1;
            }
            b'\r' => {
                count += 1;
                index += 1;
                if bytes.get(index) == Some(&b'\n') {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use std::{format, string::String, vec};

    use crate::{ErrorKind, Item, StringKind, TokenKind, TriviaKind, lex};

    #[test]
    fn preserves_comment_and_suffix_comment_order() {
        let stream = lex("key: 1 // comment\nnext: 2\n").unwrap();

        assert_eq!(
            describe_items(&stream.items),
            vec![
                "token Identifier \"key\"",
                "token Colon \":\"",
                "trivia Whitespace \" \" lines=0",
                "token Number \"1\"",
                "trivia Whitespace \" \" lines=0",
                "trivia Comment \"// comment\" lines=0",
                "trivia Whitespace \"\\n\" lines=1",
                "token Identifier \"next\"",
                "token Colon \":\"",
                "trivia Whitespace \" \" lines=0",
                "token Number \"2\"",
                "trivia Whitespace \"\\n\" lines=1",
            ]
        );
    }

    #[test]
    fn lexes_all_string_families() {
        let stream = lex("\"a\" 'b' \"\"\"c\"\"\" '''d'''").unwrap();

        let strings: Vec<_> = stream
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Token(token) => Some((token.kind, token.raw)),
                Item::Trivia(_) => None,
            })
            .collect();

        assert_eq!(
            strings,
            vec![
                (TokenKind::String(StringKind::Basic), "\"a\""),
                (TokenKind::String(StringKind::Literal), "'b'"),
                (
                    TokenKind::String(StringKind::MultilineBasic),
                    "\"\"\"c\"\"\""
                ),
                (TokenKind::String(StringKind::MultilineLiteral), "'''d'''"),
            ]
        );
    }

    #[test]
    fn lexes_maps_lists_and_variants() {
        let stream = lex("map: { items: [Rgb(1)] }").unwrap();

        let tokens: Vec<_> = stream
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Token(token) => Some((token.kind, token.raw)),
                Item::Trivia(_) => None,
            })
            .collect();

        assert_eq!(
            tokens,
            vec![
                (TokenKind::Identifier, "map"),
                (TokenKind::Colon, ":"),
                (TokenKind::OpenBrace, "{"),
                (TokenKind::Identifier, "items"),
                (TokenKind::Colon, ":"),
                (TokenKind::OpenList, "["),
                (TokenKind::Identifier, "Rgb"),
                (TokenKind::OpenParen, "("),
                (TokenKind::Number, "1"),
                (TokenKind::CloseParen, ")"),
                (TokenKind::CloseList, "]"),
                (TokenKind::CloseBrace, "}"),
            ]
        );
    }

    #[test]
    fn counts_crlf_as_single_line_breaks() {
        let stream = lex("a\r\n\r\nb").unwrap();

        let whitespace = stream
            .items
            .iter()
            .find_map(|item| match item {
                Item::Trivia(trivia) if trivia.kind == TriviaKind::Whitespace => Some(*trivia),
                _ => None,
            })
            .unwrap();

        assert_eq!(whitespace.raw, "\r\n\r\n");
        assert_eq!(whitespace.line_breaks, 2);
    }

    #[test]
    fn rejects_unterminated_string() {
        let err = lex("\"unterminated").unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnterminatedString);
    }

    #[test]
    fn rejects_hidden_unicode_in_comment() {
        let input = format!("// {}\nkey", '\u{11101}');
        let err = lex(&input).unwrap_err();
        assert!(matches!(err.kind, ErrorKind::DisallowedInvisibleUnicode(_)));
    }

    fn describe_items(items: &[Item<'_>]) -> Vec<String> {
        items
            .iter()
            .map(|item| match item {
                Item::Token(token) => format!("token {:?} {:?}", token.kind, token.raw),
                Item::Trivia(trivia) => format!(
                    "trivia {:?} {:?} lines={}",
                    trivia.kind, trivia.raw, trivia.line_breaks
                ),
            })
            .collect()
    }
}
