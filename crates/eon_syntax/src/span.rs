/// The byte range of something in the source code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl ariadne::Span for Span {
    type SourceId = ();

    #[inline]
    fn source(&self) -> &Self::SourceId {
        &()
    }

    #[inline]
    fn start(&self) -> usize {
        self.start
    }

    #[inline]
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
