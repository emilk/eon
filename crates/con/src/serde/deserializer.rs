// See https://serde.rs/impl-deserializer.html

use logos::source;
use serde::de::{self, Error as _, Visitor};

use crate::{
    Number, Value,
    ast::{AstValue, CommentedKeyValue, CommentedValue},
    span::Span,
};

// TODO: include spans and rich error messages
#[derive(Debug, Clone)]
pub struct DeserErrror {
    pub msg: String,
    pub span: Option<Span>,
}

impl DeserErrror {
    pub fn into_error(self, source: &str) -> crate::Error {
        let Self { msg, span } = self;
        if let Some(span) = span {
            crate::Error::new_at(source, span, msg)
        } else {
            crate::Error::custom(msg)
        }
    }
}

impl std::error::Error for DeserErrror {}

impl std::fmt::Display for DeserErrror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            panic!("Do not call this directlty!");
        } else {
            self.msg.fmt(f)
        }
    }
}

impl de::Error for DeserErrror {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self {
            msg: msg.to_string(),
            span: None,
        }
    }
}

type Result<T = (), E = DeserErrror> = std::result::Result<T, E>;

// ----------------------------------------------------

pub struct AstValueDeser<'de> {
    value: &'de CommentedValue<'de>,
}

impl<'de> AstValueDeser<'de> {
    pub fn new(value: &'de CommentedValue<'de>) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for &'_ mut AstValueDeser<'de> {
    type Error = DeserErrror;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut result = match &self.value.value {
            AstValue::Identifier(identifier) => match identifier.as_ref() {
                "null" => visitor.visit_unit(),
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                some_other_string => {
                    // We get here in case map keys
                    visitor.visit_borrowed_str(some_other_string)
                }
            },

            AstValue::Number(num_str) => match Number::try_parse(num_str) {
                Ok(number) => {
                    if let Some(n) = number.as_u64() {
                        visitor.visit_u64(n)
                    } else if let Some(n) = number.as_i64() {
                        visitor.visit_i64(n)
                    } else if let Some(n) = number.as_f64() {
                        visitor.visit_f64(n)
                    } else if let Some(n) = number.as_i128() {
                        visitor.visit_i128(n)
                    } else if let Some(n) = number.as_u128() {
                        visitor.visit_u128(n)
                    } else {
                        Err(DeserErrror::custom(format!("Invalid numbner: {number}")))
                    }
                }
                Err(err) => Err(DeserErrror::custom(err)),
            },

            AstValue::QuotedString(quoted) => snailquote::unescape(quoted)
                .map_err(|err| {
                    DeserErrror::custom(format!(
                        "Failed to unescape quoted string: {quoted:?}: {err}"
                    ))
                })
                .and_then(|unescaped| visitor.visit_string(unescaped)),

            AstValue::List(list) => visitor.visit_seq(ListAccess(&list.values)),

            AstValue::Map(map) => visitor.visit_map(MapAcceses {
                kvs: &map.key_values,
            }),
        };

        if let Err(err) = &mut result {
            if err.span.is_none() {
                err.span = Some(self.value.span);
            }
        }

        result
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ListAccess<'de>(&'de [CommentedValue<'de>]);

impl<'de> de::SeqAccess<'de> for ListAccess<'de> {
    type Error = DeserErrror;

    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let [first, rest @ ..] = self.0 {
            self.0 = rest;
            seed.deserialize(&mut AstValueDeser::new(&first)).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct MapAcceses<'de> {
    kvs: &'de [CommentedKeyValue<'de>],
}

impl<'de> de::MapAccess<'de> for MapAcceses<'de> {
    type Error = DeserErrror;

    fn size_hint(&self) -> Option<usize> {
        Some(self.kvs.len())
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(kv) = self.kvs.first() {
            seed.deserialize(&mut AstValueDeser::new(&kv.key)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let [first, rest @ ..] = self.kvs {
            self.kvs = rest;
            seed.deserialize(&mut AstValueDeser::new(&first.value))
        } else {
            Err(DeserErrror::custom("No more values in map"))
        }
    }
}

// ----------------------------------------------------

pub struct ValueDeser<'de> {
    value: &'de Value,
}

impl<'de> ValueDeser<'de> {
    pub fn new(value: &'de Value) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for &'_ mut ValueDeser<'de> {
    type Error = DeserErrror;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_unit(),

            Value::Bool(b) => visitor.visit_bool(*b),

            Value::Number(number) => {
                if let Some(n) = number.as_u64() {
                    visitor.visit_u64(n)
                } else if let Some(n) = number.as_i64() {
                    visitor.visit_i64(n)
                } else if let Some(n) = number.as_f64() {
                    visitor.visit_f64(n)
                } else if let Some(n) = number.as_i128() {
                    visitor.visit_i128(n)
                } else if let Some(n) = number.as_u128() {
                    visitor.visit_u128(n)
                } else {
                    return Err(DeserErrror::custom(format!("Invalid numbner: {number}")));
                }
            }

            Value::String(string) => visitor.visit_borrowed_str(string),

            Value::List(list) => visitor.visit_seq(ValueListAccess(list)),

            Value::Map(map) => visitor.visit_map(ValueMapAcceses {
                iter: map.iter(),
                next_value: None,
            }),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ValueListAccess<'de>(&'de [Value]);

impl<'de> de::SeqAccess<'de> for ValueListAccess<'de> {
    type Error = DeserErrror;

    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let [first, rest @ ..] = self.0 {
            self.0 = rest;
            seed.deserialize(&mut ValueDeser { value: first }).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct ValueMapAcceses<'de, I>
where
    I: Iterator<Item = (&'de String, &'de Value)>,
{
    iter: I,
    next_value: Option<&'de Value>,
}

impl<'de> de::MapAccess<'de> for ValueMapAcceses<'de, indexmap::map::Iter<'de, String, Value>> {
    type Error = DeserErrror;

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.size_hint().0)
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.next_value = Some(value);
            seed.deserialize(&mut MapKeyDeser { key }).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(value) = self.next_value.take() {
            seed.deserialize(&mut ValueDeser { value })
        } else {
            Err(DeserErrror::custom("No more values in map"))
        }
    }
}

struct MapKeyDeser<'de> {
    key: &'de String,
}

impl<'de> de::Deserializer<'de> for &'_ mut MapKeyDeser<'de> {
    type Error = DeserErrror;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.key)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
