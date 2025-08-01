mod deserialize_value;
mod deserializer;
mod serializer;

use serde::Serialize;

use crate::{FormatOptions, Value};

use self::serializer::Serializer;

pub use self::serializer::SerializationError;

/// Serialize a value (using serde) into a [`Value`].
///
/// ## Example
/// ```rust
/// #[derive(serde::Serialize)]
/// struct Config {
///     string: String,
///     age: u32,
/// }
///
/// let config = Config {
///     string: "Hello Eon!".to_string(),
///     age: 42,
/// };
///
/// let value = eon::to_value(&config).unwrap();
///
/// let expected = eon::Value::Map([
///     ("string".to_string(), eon::Value::from("Hello Eon!")),
///     ("age".to_string(), eon::Value::from(42)),
/// ].into_iter().collect());
///
/// assert_eq!(value, expected);
/// ```
pub fn to_value<T>(value: &T) -> Result<Value, SerializationError>
where
    T: ?Sized + Serialize,
{
    let serializer = Serializer::default();
    value.serialize(&serializer)
}

/// Serialize a value (using serde) into an Eon string.
///
/// ## Example
/// ```rust
/// #[derive(serde::Serialize)]
/// struct Config {
///     string: String,
///     age: u32,
/// }
///
/// let config = Config {
///     string: "Hello Eon!".to_string(),
///     age: 42,
/// };
///
/// let eon_string = eon::to_string(&config, &eon::FormatOptions::default()).unwrap();
///
/// assert_eq!(eon_string.trim(), r#"
/// string: "Hello Eon!"
/// age: 42
/// "#.trim());
/// ```
pub fn to_string<T>(value: &T, options: &FormatOptions) -> Result<String, SerializationError>
where
    T: Serialize,
{
    to_value(value).map(|value| value.format(options))
}

/// Parse an Eon value from a string into a type `T` that implements [`serde::de::DeserializeOwned`].
///
/// ## Example
/// ```rust
/// #[derive(serde::Deserialize)]
/// struct Config {
///     string: String,
///     age: u32,
/// }
///
/// let eon_source = r#"
///     string: "Hello Eon!"
///     age: 42
/// "#;
///
/// let config: Config = eon::from_str(eon_source).unwrap();
///
/// assert_eq!(config.string, "Hello Eon!");
/// assert_eq!(config.age, 42);
/// ```
pub fn from_str<T>(eon_source: &str) -> Result<T, crate::Error>
where
    T: serde::de::DeserializeOwned,
{
    eon_syntax::TokenTree::parse_str(eon_source).and_then(|token_tree| {
        let deser = self::deserializer::TokenTreeDeserializer::new(&token_tree);
        T::deserialize(deser).map_err(|err| err.into_error(eon_source))
    })
}
