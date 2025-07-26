use crate::span::Span;

/// Represent an error during parsing
type ErrorReport = ariadne::Report<'static, Span>;

pub enum Error {
    Custom {
        msg: String,
    },
    At {
        source: ariadne::Source,
        span: Span,
        message: String,
    },
}

impl Error {
    pub fn new_at(con_source: &str, span: Span, message: impl Into<String>) -> Self {
        Self::At {
            source: ariadne::Source::from(con_source.to_owned()),
            span,
            message: message.into(),
        }
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            msg: message.into(),
        }
    }

    pub fn to_string_with_color(&self, color: bool) -> String {
        match self {
            Self::Custom { msg } => msg.to_owned(),
            Self::At {
                source,
                span,
                message,
            } => {
                let report = ErrorReport::build(ariadne::ReportKind::Error, *span)
                    .with_label(ariadne::Label::new(*span).with_message(message))
                    .with_config(ariadne::Config::default().with_color(color))
                    .finish();

                let mut utf8 = vec![];
                let mut cursor = std::io::Cursor::new(&mut utf8);
                match report.write(source, &mut cursor) {
                    Ok(_) => {
                        strip_trailing_whitespace_on_each_line(&String::from_utf8_lossy(&utf8))
                    }
                    Err(_) => message.to_owned(),
                }
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string_with_color(false).fmt(f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string_with_color(false).fmt(f)
    }
}

impl std::error::Error for Error {}

/// A type alias for a result that uses the [`Error`] type defined above.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

fn strip_trailing_whitespace_on_each_line(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}
