mod deserialize_value;
mod deserializer;
mod serialize_value;
mod serializer;

use serde::Serialize;

use crate::{FormatOptions, Value, ast::CommentedValue};

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

pub fn from_str<T>(source: &str) -> Result<T, crate::Error>
where
    T: serde::de::DeserializeOwned,
{
    CommentedValue::parse_str(source).and_then(|commented_value| {
        let mut deser = self::deserializer::AstValueDeser::new(&commented_value.value);
        T::deserialize(&mut deser).map_err(|err| unimplemented!())
    })
}
