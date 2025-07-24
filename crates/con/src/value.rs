use crate::Result;

/// Represents any Con value.
///
/// This does NOT include comments.
/// For that, use [`crate::ast::CommentedValue`].
pub enum Value {
    /// Special `null` value
    Null,

    /// `true` or `false`
    Bool(bool),

    /// An integer or floating point number
    Number(Number),

    /// The contents of a string, i.e. without surrounding quotes.
    String(String),

    /// A list of values.
    List(Vec<Value>),

    /// Maps strings to values, i.e. like a `struct`.
    Object(Object),
}

impl Number {
    pub(crate) fn try_parse(source: &str, string: &str) -> Result<Self> {
        Ok(Number(NumberImpl::String(string.to_owned())))
    }
}

/// Represents a number (float, integer, …)
pub struct Number(NumberImpl);

enum NumberImpl {
    F64(f64),

    /// Yet-to-be parsed.
    String(String),
}

/// Maps strings to values, i.e. like a `struct`.
pub type Object = indexmap::IndexMap<String, Value>;
