mod map;
mod number;
mod variant;

use eon_syntax::{FormatOptions, Result, TokenTree};

pub use self::{map::Map, number::Number, variant::Variant};

/// Represents any Eon value.
///
/// Load an Eon document into a [`Value`] using [`Value::from_str`](std::str::FromStr::from_str).
/// Serialize a [`Value`] into an Eon string using [`Value::format`].
///
/// ## See also
/// A [`Value`] does NOT include comments.
/// For that, use [`eon_syntax::TokenTree`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    /// Special `null` value
    Null,

    /// `true` or `false`
    Bool(bool),

    /// An integer or floating point number
    Number(Number),

    /// A string value, like `"Hello, world!"`
    ///
    /// Also commonly used as the key in a [`Map`].
    ///
    /// Strings are also used for simple sum-type (enum) variants values, e.g. `"Maybe"`.
    /// See [`Self::Variant`] for more complex sum-type (enum) variants.
    String(String),

    /// A list of values.
    List(Vec<Value>),

    /// Maps strings to values, i.e. like a `struct`.
    Map(Map),

    /// A sum-type (enum) variant containing some data, like `"Rgb"(255, 0, 0)`.
    ///
    /// For simple enum types (e.g. `enum Maybe { Yes, No }`),
    /// the variants will be represented as [`Self::String`] instead.
    Variant(Variant),
}

impl Value {
    /// Construct a variant of an enum (sum-type) with a name and optional values.
    ///
    /// If the values is empty, this will return a [`Value::String`],
    /// otherwise it will return a [`Value::Variant`].
    pub fn new_variant(name: String, values: Vec<Self>) -> Self {
        if let Ok(values) = vec1::Vec1::try_from_vec(values) {
            Self::Variant(Variant { name, values })
        } else {
            Self::String(name)
        }
    }

    /// Pretty-print a [`Value`] to an Eon string.
    ///
    /// You can parse the result with [`Value::from_str`](std::str::FromStr::from_str).
    pub fn format(&self, options: &FormatOptions) -> String {
        TokenTree::from(self.clone()).format(options)
    }

    /// Return the bool value iff this is a [`Value::Bool`].
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Return the number iff this is a [`Value::Number`].
    pub fn as_number(&self) -> Option<&Number> {
        if let Self::Number(n) = self {
            Some(n)
        } else {
            None
        }
    }

    /// Return the string iff this is a [`Value::String`].
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Return the list iff this is a [`Value::List`].
    pub fn as_list(&self) -> Option<&[Self]> {
        if let Self::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Return the map iff this is a [`Value::Map`].
    pub fn as_map(&self) -> Option<&Map> {
        if let Self::Map(m) = self {
            Some(m)
        } else {
            None
        }
    }

    /// Return the variant iff this is a [`Value::Variant`].
    pub fn as_variant(&self) -> Option<&Variant> {
        if let Self::Variant(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(&crate::FormatOptions::default()).fmt(f)
    }
}

impl std::str::FromStr for Value {
    type Err = crate::Error;

    fn from_str(eon_source: &str) -> Result<Self, Self::Err> {
        TokenTree::parse_str(eon_source).and_then(|tt| Self::try_from_token_tree(eon_source, &tt))
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

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

impl From<Vec<Self>> for Value {
    #[inline]
    fn from(value: Vec<Self>) -> Self {
        Self::List(value)
    }
}

impl From<Map> for Value {
    #[inline]
    fn from(value: Map) -> Self {
        Self::Map(value)
    }
}

impl From<Variant> for Value {
    #[inline]
    fn from(value: Variant) -> Self {
        Self::Variant(value)
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

impl From<char> for Value {
    #[inline]
    fn from(value: char) -> Self {
        Self::String(value.to_string())
    }
}

impl From<&[u8]> for Value {
    #[inline]
    fn from(value: &[u8]) -> Self {
        // TODO(emilk): encode byte arrays more efficiently? Maybe a special byte-hex-encoding, like `b"ab01…"`?
        Self::List(value.iter().map(|&b| Self::from(b)).collect())
    }
}
