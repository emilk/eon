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

/// Maps strings to values, i.e. like a `struct`.
// TODO: consider adding an optional name for objects which can be used for tuple variants?
pub type Object = indexmap::IndexMap<String, Value>; // TODO: Value to Value

/// Represents a number (float, integer, …)
pub struct Number(NumberImpl);

enum NumberImpl {
    I128(i128),
    U128(u128),

    // Having this seperatedly allows us to encode f32 using less precision.
    F32(f32),

    F64(f64),

    /// Yet-to-be parsed.
    String(String),
}

impl Number {
    pub(crate) fn try_parse(source: &str, string: &str) -> Result<Self> {
        Ok(Self(NumberImpl::String(string.to_owned())))
    }
}

impl From<i8> for Number {
    #[inline]
    fn from(value: i8) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i16> for Number {
    #[inline]
    fn from(value: i16) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i32> for Number {
    #[inline]
    fn from(value: i32) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i64> for Number {
    #[inline]
    fn from(value: i64) -> Self {
        Self(NumberImpl::I128(value as _))
    }
}

impl From<i128> for Number {
    #[inline]
    fn from(value: i128) -> Self {
        Self(NumberImpl::I128(value))
    }
}

impl From<u8> for Number {
    #[inline]
    fn from(value: u8) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u16> for Number {
    #[inline]
    fn from(value: u16) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u32> for Number {
    #[inline]
    fn from(value: u32) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u64> for Number {
    #[inline]
    fn from(value: u64) -> Self {
        Self(NumberImpl::U128(value as _))
    }
}

impl From<u128> for Number {
    #[inline]
    fn from(value: u128) -> Self {
        Self(NumberImpl::U128(value))
    }
}

impl From<f32> for Number {
    #[inline]
    fn from(value: f32) -> Self {
        Self(NumberImpl::F32(value))
    }
}

impl From<f64> for Number {
    #[inline]
    fn from(value: f64) -> Self {
        Self(NumberImpl::F64(value))
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            NumberImpl::I128(n) => n.fmt(f),

            NumberImpl::U128(n) => n.fmt(f),

            NumberImpl::F32(n) => {
                if n.is_nan() {
                    write!(f, "NaN")
                } else if *n == f32::NEG_INFINITY {
                    write!(f, "-inf")
                } else if *n == f32::INFINITY {
                    write!(f, "+inf")
                } else {
                    // TODO: always include a decimal point?
                    n.fmt(f)
                }
            }

            NumberImpl::F64(n) => {
                if n.is_nan() {
                    write!(f, "NaN")
                } else if *n == f64::NEG_INFINITY {
                    write!(f, "-inf")
                } else if *n == f64::INFINITY {
                    write!(f, "+inf")
                } else {
                    // TODO: always include a decimal point?
                    n.fmt(f)
                }
            }

            NumberImpl::String(s) => s.fmt(f),
        }
    }
}
