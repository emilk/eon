pub mod ast;
pub mod format;
pub mod parse;
pub mod span;
pub mod token;

/// Represent an error during parsing
pub type Error = ariadne::Report<'static, span::Span>; // TODO: box, since this is huge

/// A type alias for a result that uses the [`Error`] type defined above.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

pub use crate::format::FormatOptions;

/// Parses a Con file and re-indents and formats it in a pretty way.
pub fn reformat(input: &str, options: &FormatOptions) -> Result<String> {
    let value = parse::parse_top_str(input)?;
    Ok(format::format(&value, options))
}
