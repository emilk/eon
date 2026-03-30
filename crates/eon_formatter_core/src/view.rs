use crate::{
    Item, Token, TokenStream, Trivia, TriviaKind,
};

/// Iterate over the syntax tokens in a [`TokenStream`].
pub struct TokenRefs<'stream, 'source> {
    stream: &'stream TokenStream<'source>,
    next_token_index: usize,
}

impl<'stream, 'source> TokenRefs<'stream, 'source> {
    pub(crate) fn new(stream: &'stream TokenStream<'source>) -> Self {
        Self {
            stream,
            next_token_index: 0,
        }
    }
}

impl<'stream, 'source> Iterator for TokenRefs<'stream, 'source> {
    type Item = TokenRef<'stream, 'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let token_index = self.next_token_index;
        if token_index >= self.stream.token_item_indices.len() {
            return None;
        }
        self.next_token_index += 1;
        Some(TokenRef {
            stream: self.stream,
            token_index,
        })
    }
}

/// Borrowed view of one syntax token with access to its surrounding trivia.
#[derive(Clone, Copy)]
pub struct TokenRef<'stream, 'source> {
    stream: &'stream TokenStream<'source>,
    token_index: usize,
}

impl<'stream, 'source> TokenRef<'stream, 'source> {
    pub(crate) fn new(stream: &'stream TokenStream<'source>, token_index: usize) -> Self {
        Self { stream, token_index }
    }

    /// Zero-based token index within the token stream.
    pub fn index(self) -> usize {
        self.token_index
    }

    /// The underlying syntax token.
    pub fn token(self) -> Token<'source> {
        let item_index = self.stream.token_item_indices[self.token_index];
        let Item::Token(token) = self.stream.items[item_index] else {
            unreachable!("token_item_indices always point at tokens");
        };
        token
    }

    /// Trivia between the previous syntax token and this one.
    pub fn leading_trivia(self) -> TriviaIter<'stream, 'source> {
        let start = if self.token_index == 0 {
            0
        } else {
            self.stream.token_item_indices[self.token_index - 1] + 1
        };
        let end = self.stream.token_item_indices[self.token_index];
        TriviaIter::new(&self.stream.items, start, end)
    }

    /// Trivia between this syntax token and the next one.
    pub fn trailing_trivia(self) -> TriviaIter<'stream, 'source> {
        let start = self.stream.token_item_indices[self.token_index] + 1;
        let end = self
            .stream
            .token_item_indices
            .get(self.token_index + 1)
            .copied()
            .unwrap_or(self.stream.items.len());
        TriviaIter::new(&self.stream.items, start, end)
    }

    /// Returns `true` when the token is preceded by one or more comment items.
    pub fn has_leading_comments(self) -> bool {
        self.leading_trivia()
            .any(|trivia| trivia.kind == TriviaKind::Comment)
    }

    /// Classify the layout before this token for formatting decisions.
    pub fn leading_trivia_kind(self) -> LeadingTriviaKind {
        let mut saw_line_break = false;

        for trivia in self.leading_trivia() {
            if trivia.kind == TriviaKind::Whitespace && trivia.line_breaks >= 2 {
                return LeadingTriviaKind::BlankLine;
            }
            if trivia.line_breaks > 0 {
                saw_line_break = true;
            }
        }

        if saw_line_break {
            LeadingTriviaKind::Newline
        } else {
            LeadingTriviaKind::Inline
        }
    }

    /// Returns the inline `// comment` attached to the token, if any.
    pub fn suffix_comment(self) -> Option<Trivia<'source>> {
        for trivia in self.trailing_trivia() {
            match trivia.kind {
                TriviaKind::Whitespace if trivia.line_breaks > 0 => return None,
                TriviaKind::Comment => return Some(trivia),
                TriviaKind::Whitespace => {}
            }
        }
        None
    }
}

/// Iterate over trivia items in a contiguous source region.
#[derive(Clone)]
pub struct TriviaIter<'stream, 'source> {
    items: &'stream [Item<'source>],
    next_item_index: usize,
    end_item_index: usize,
}

impl<'stream, 'source> TriviaIter<'stream, 'source> {
    fn new(items: &'stream [Item<'source>], start_item_index: usize, end_item_index: usize) -> Self {
        Self {
            items,
            next_item_index: start_item_index,
            end_item_index,
        }
    }
}

impl<'stream, 'source> Iterator for TriviaIter<'stream, 'source> {
    type Item = Trivia<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_item_index < self.end_item_index {
            let item_index = self.next_item_index;
            self.next_item_index += 1;

            if let Item::Trivia(trivia) = self.items[item_index] {
                return Some(trivia);
            }
        }

        None
    }
}

/// Formatter-relevant layout classification for trivia before a token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeadingTriviaKind {
    /// No line break before the token.
    Inline,
    /// At least one line break before the token, but no blank line.
    Newline,
    /// An empty line appears before the token.
    BlankLine,
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::{LeadingTriviaKind, TriviaKind, lex};

    #[test]
    fn token_refs_expose_leading_and_trailing_trivia() {
        let stream = lex("// first\nkey: 1 // suffix\n\nnext: 2").unwrap();
        let tokens: Vec<_> = stream.tokens().collect();

        let key = tokens[0];
        assert_eq!(key.token().raw, "key");
        assert!(key.has_leading_comments());
        assert_eq!(key.leading_trivia_kind(), LeadingTriviaKind::Newline);
        assert!(key.suffix_comment().is_none());

        let one = tokens[2];
        assert_eq!(one.token().raw, "1");
        assert_eq!(one.suffix_comment().map(|comment| comment.raw), Some("// suffix"));

        let next = tokens[3];
        assert_eq!(next.token().raw, "next");
        assert_eq!(next.leading_trivia_kind(), LeadingTriviaKind::BlankLine);
    }

    #[test]
    fn suffix_comment_stops_at_newline() {
        let stream = lex("value\n// next line\nother").unwrap();
        let token = stream.tokens().next().unwrap();

        assert_eq!(token.suffix_comment(), None);
    }

    #[test]
    fn token_stream_counts_tokens() {
        let stream = lex("alpha: [1 2]").unwrap();
        assert_eq!(stream.token_count(), 6);
        assert!(!stream.has_no_tokens());
    }

    #[test]
    fn trailing_trivia_iterator_returns_only_trivia() {
        let stream = lex("value // suffix\n").unwrap();
        let token = stream.tokens().next().unwrap();
        let trailing: Vec<_> = token.trailing_trivia().collect();

        assert_eq!(trailing.len(), 3);
        assert_eq!(trailing[1].kind, TriviaKind::Comment);
        assert_eq!(trailing[1].raw, "// suffix");
    }
}
