use crate::Value;

/// A sum-type (enum) variant, such as `"Black"` or `"Rgb"(255, 0, 0)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variant {
    /// The name of the variant, like `Rgb`.
    pub name: String,

    /// The contents of the variant.
    ///
    /// This can be empty for unit variants.
    pub values: Vec<Value>,
}
