use vec1::Vec1;

use crate::Value;

/// A sum-type (enum) variant containing some data, like `"Rgb"(255, 0, 0)`.
///
/// For simple enum types (e.g. `enum Maybe { Yes, No }`),
/// the variants will be represented as [`Value::String`] instead.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variant {
    /// The name of the variant, like `Rgb`.
    pub name: String,

    /// The contents of the variant.
    ///
    /// Note that this cannot be empty.
    /// A variant with no contents is represented as a [`Value::String`].
    pub values: Vec1<Value>,
}
