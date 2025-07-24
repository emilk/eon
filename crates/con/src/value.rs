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

/// Maps strings to values, i.e. like a `struct`.
pub type Object = indexmap::IndexMap<String, Value>;

/// Represents a number (float, integer, …)
pub struct Number(NumberImpl);

enum NumberImpl {
    I64(i64),

    F64(f64),

    /// Yet-to-be parsed.
    String(String),
}

impl Number {
    pub(crate) fn try_parse(source: &str, string: &str) -> Result<Self> {
        Ok(Number(NumberImpl::String(string.to_owned())))
    }
}

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Self(NumberImpl::I64(value as _))
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self(NumberImpl::F64(value))
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            NumberImpl::I64(n) => n.fmt(f),
            NumberImpl::F64(n) => n.fmt(f), // TODO: always include a decimal point. TODO: format inf/nan.
            NumberImpl::String(s) => s.fmt(f),
        }
    }
}
