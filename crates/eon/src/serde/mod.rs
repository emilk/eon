mod deserialize_value;
mod deserializer;
mod serializer;

use serde::Serialize;

use crate::{FormatOptions, Value};

use self::serializer::Serializer;

pub use self::serializer::SerializationError;

/// Serialize a value (using serde) into a [`Value`].
pub fn to_value<T>(value: &T) -> Result<Value, SerializationError>
where
    T: ?Sized + Serialize,
{
    let serializer = Serializer::default();
    value.serialize(&serializer)
}

/// Serialize a value (using serde) into an Eon string.
pub fn to_string<T>(value: &T, options: &FormatOptions) -> Result<String, SerializationError>
where
    T: Serialize,
{
    to_value(value).map(|value| value.format(options))
}

/// Parse an Eon value from a string into a type `T` that implements [`serde::de::DeserializeOwned`].
pub fn from_str<T>(eon_source: &str) -> Result<T, crate::Error>
where
    T: serde::de::DeserializeOwned,
{
    eon_syntax::TokenTree::parse_str(eon_source).and_then(|token_tree| {
        let deser = self::deserializer::TokenTreeDeserializer::new(&token_tree);
        T::deserialize(deser).map_err(|err| err.into_error(eon_source))
    })
}
