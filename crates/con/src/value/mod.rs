mod map;
mod number;

pub use self::{map::Map, number::Number};

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
    Map(Map),
}

impl From<bool> for Value {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

macro_rules! impl_value_from_number {
    ($t:ty) => {
        impl From<$t> for Value {
            #[inline]
            fn from(value: $t) -> Self {
                Value::Number(Number::from(value))
            }
        }
    };
}

impl_value_from_number!(i8);
impl_value_from_number!(i16);
impl_value_from_number!(i32);
impl_value_from_number!(i64);
impl_value_from_number!(i128);
impl_value_from_number!(u8);
impl_value_from_number!(u16);
impl_value_from_number!(u32);
impl_value_from_number!(u64);
impl_value_from_number!(u128);
impl_value_from_number!(f32);
impl_value_from_number!(f64);

impl From<Number> for Value {
    #[inline]
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Value {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<char> for Value {
    #[inline]
    fn from(value: char) -> Self {
        Self::String(value.to_string())
    }
}

impl From<&[u8]> for Value {
    #[inline]
    fn from(value: &[u8]) -> Self {
        // TODO: something more efficient?
        // Maybe a special byte-hex-encoding?
        Self::List(value.iter().map(|&b| Self::from(b)).collect())
    }
}
