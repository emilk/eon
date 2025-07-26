/// The byte range of something in the source code.
#[derive(Clone, Copy, Debug, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
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

impl std::ops::BitOr for Span {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
