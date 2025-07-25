use crate::span::Span;

/// Represent an error during parsing
pub type ErrorReport = ariadne::Report<'static, Span>;

pub fn error_report_at(span: Span, message: impl Into<String>) -> ErrorReport {
    ErrorReport::build(ariadne::ReportKind::Error, span)
        .with_label(ariadne::Label::new(span).with_message(message.into()))
        .finish()
}

#[derive(Clone)]
pub enum Error {
    Custom {
        msg: String,
    },
    AtSource {
        source: ariadne::Source,

        /// Wrapped in a box to because [`ariadne::Report`] is huge, and not `Clone`.
        report: std::rc::Rc<ErrorReport>,
    },
}

impl Error {
    pub fn new(source: &str, report: ErrorReport) -> Self {
        Self::AtSource {
            source: ariadne::Source::from(source.to_owned()),
            report: std::rc::Rc::new(report),
        }
    }

    pub fn new_at(source: &str, span: Span, message: impl Into<String>) -> Self {
        Self::new(source, error_report_at(span, message))
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            msg: message.into(),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom { msg } => msg.fmt(f),
            Self::AtSource { source, report } => {
                let mut utf8 = vec![];
                let mut cursor = std::io::Cursor::new(&mut utf8);
                match report.write(source, &mut cursor) {
                    Ok(_) => {
                        write!(f, "{}", String::from_utf8_lossy(&utf8))
                    }
                    Err(_) => report.fmt(f),
                }
            }
        }
    }
}

/// A type alias for a result that uses the [`Error`] type defined above.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;
