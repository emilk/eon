mod deserialize_value;
mod deserializer;
mod serialize_value;
mod serializer;

use serde::Serialize;

use crate::{FormatOptions, Value, token_tree::TokenTree};

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

/// Serialize a value (using serde) into a Con string.
pub fn to_string<T>(value: &T, options: &FormatOptions) -> Result<String, SerializationError>
where
    T: Serialize,
{
    to_value(value).map(|value| value.format(options))
}

/// Parse a Con value from a string into a type `T` that implements [`serde::de::DeserializeOwned`].
pub fn from_str<T>(source: &str) -> Result<T, crate::Error>
where
    T: serde::de::DeserializeOwned,
{
    TokenTree::parse_str(source).and_then(|commented_value| {
        let deser = self::deserializer::TokenTreeDeserializer::new(&commented_value);
        T::deserialize(deser).map_err(|err| err.into_error(source))
    })
}
