use crate::span::Span;

/// Represent an error during parsing
pub type ErrorReport = ariadne::Report<'static, Span>;

pub fn error_report_at(span: Span, message: impl Into<String>) -> ErrorReport {
    ErrorReport::build(ariadne::ReportKind::Error, span)
        .with_label(ariadne::Label::new(span).with_message(message.into()))
        .finish()
}

#[derive(Clone)]
pub struct Error {
    source: ariadne::Source,

    /// Wrapped in a box to because [`ariadne::Report`] is huge, and not `Clone`.
    report: std::rc::Rc<ErrorReport>,
}

impl Error {
    pub fn new(source: &str, report: ErrorReport) -> Self {
        Self {
            source: ariadne::Source::from(source.to_owned()),
            report: std::rc::Rc::new(report),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut utf8 = vec![];
        let mut cursor = std::io::Cursor::new(&mut utf8);
        self.report.write(&self.source, &mut cursor).unwrap();
        write!(f, "{}", String::from_utf8_lossy(&utf8))
    }
}

/// A type alias for a result that uses the [`Error`] type defined above.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;
