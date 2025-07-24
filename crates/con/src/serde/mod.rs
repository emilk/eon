mod deserialize_value;
mod deserializer;
mod serializer;

use serde::Serialize;

use crate::{FormatOptions, Value};

use self::serializer::Serializer;

pub use self::serializer::SerializationError;

pub fn to_value<T>(value: &T) -> Result<Value, SerializationError>
where
    T: ?Sized + Serialize,
{
    let serializer = Serializer::default();
    value.serialize(&serializer)
}

pub fn to_string<T>(value: &T, options: &FormatOptions) -> Result<String, SerializationError>
where
    T: Serialize,
{
    to_value(value).map(|value| value.format(options))
}
