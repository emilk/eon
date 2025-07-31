// See https://serde.rs/impl-deserializer.html

use std::str::FromStr as _;

use serde::{
    Deserializer as _,
    de::{self, Error as _, Visitor},
};

use crate::Number;

use eon_syntax::{Span, TokenKeyValue, TokenTree, TokenValue, unescape_and_unquote};

#[derive(Debug, Clone)]
pub struct DeserError {
    pub msg: String,
    pub span: Option<Span>,
}

impl DeserError {
    pub fn new(span: Option<Span>, msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            span,
        }
    }

    pub fn into_error(self, eon_source: &str) -> crate::Error {
        let Self { msg, span } = self;
        if let Some(span) = span {
            crate::Error::new_at(eon_source, span, msg)
        } else {
            crate::Error::custom(msg)
        }
    }
}

impl std::error::Error for DeserError {}

impl std::fmt::Display for DeserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(debug_assertions) {
            panic!("Do not call this directlty!");
        } else {
            self.msg.fmt(f)
        }
    }
}

impl de::Error for DeserError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self {
            msg: msg.to_string(),
            span: None,
        }
    }
}

type Result<T = (), E = DeserError> = std::result::Result<T, E>;

// ----------------------------------------------------

/// Consumes a [`TokenTree`] and "deserializes" it into a value that implements
/// [`serde::de::Deserialize`] (e.g. has `#[derive(serde::Deserialize)]` on it).
pub struct TokenTreeDeserializer<'de> {
    value: &'de TokenTree<'de>,
}

impl<'de> TokenTreeDeserializer<'de> {
    pub fn new(value: &'de TokenTree<'de>) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for TokenTreeDeserializer<'de> {
    type Error = DeserError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let span = self.value.span;

        let mut result = match &self.value.value {
            TokenValue::Identifier(identifier) => match identifier.as_ref() {
                "null" => visitor.visit_unit(),
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                some_other_string => {
                    // We get here in case of map keys
                    visitor.visit_borrowed_str(some_other_string)
                }
            },

            TokenValue::Number(num_str) => match Number::from_str(num_str) {
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
                        Err(DeserError::new(span, format!("Invalid numbner: {number}")))
                    }
                }
                Err(err) => Err(DeserError::new(span, err)),
            },

            TokenValue::QuotedString(quoted) => unescape_and_unquote(quoted)
                .map_err(|err| {
                    DeserError::new(
                        span,
                        format!("Failed to unescape quoted string: {quoted:?}: {err}"),
                    )
                })
                .and_then(|unescaped| visitor.visit_string(unescaped)),

            TokenValue::List(list) => visitor.visit_seq(ListAccessor(&list.values)),

            TokenValue::Map(map) => visitor.visit_map(MapAccessor {
                kvs: &map.key_values,
            }),

            TokenValue::Variant(_) => Err(DeserError::new(span, "Did not expect a variant here")),
        };

        if let Err(err) = &mut result {
            err.span = err.span.or(self.value.span);
        }

        result
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let TokenValue::Identifier(identifier) = &self.value.value {
            if identifier == "null" {
                return visitor.visit_none();
            }
        }

        visitor.visit_some(self)
    }

    fn deserialize_enum<V>(
        self,
        _enum_name: &'static str,
        variant_names: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let quoted_name;
        let values;

        match &self.value.value {
            TokenValue::QuotedString(quoted) => {
                quoted_name = quoted;
                values = &[][..];
            }
            TokenValue::Variant(variant) => {
                quoted_name = &variant.quoted_name;
                values = variant.values.as_slice();
            }
            _ => {
                return Err(DeserError::new(
                    self.value.span,
                    format!(
                        "Expected a variant name here; one of: {variant_names:?}. Got: {:?}",
                        self.value.value
                    ),
                ));
            }
        }

        let unquoted_name = unescape_and_unquote(quoted_name).map_err(|err| {
            DeserError::new(
                self.value.span,
                format!("Failed to unescape quoted name: {quoted_name:?}: {err}"),
            )
        })?;

        let name = variant_names.iter().find(|&&name| name == unquoted_name);

        let Some(name) = name else {
            return Err(DeserError::new(
                self.value.span,
                format!("Expected one of: {variant_names:?}, got: {quoted_name}"),
            ));
        };

        visitor.visit_enum(EnumAccessor {
            name_span: self.value.span,
            name,
            values,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct ListAccessor<'de>(&'de [TokenTree<'de>]);

impl<'de> de::SeqAccess<'de> for ListAccessor<'de> {
    type Error = DeserError;

    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let [first, rest @ ..] = self.0 {
            self.0 = rest;
            seed.deserialize(TokenTreeDeserializer::new(first))
                .map(Some)
        } else {
            Ok(None)
        }
    }
}

struct MapAccessor<'de> {
    kvs: &'de [TokenKeyValue<'de>],
}

impl<'de> de::MapAccess<'de> for MapAccessor<'de> {
    type Error = DeserError;

    fn size_hint(&self) -> Option<usize> {
        Some(self.kvs.len())
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(kv) = self.kvs.first() {
            seed.deserialize(TokenTreeDeserializer::new(&kv.key))
                .map(Some)
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
            seed.deserialize(TokenTreeDeserializer::new(&first.value))
        } else {
            Err(DeserError::custom("No more values in map"))
        }
    }
}

struct EnumAccessor<'de> {
    name_span: Option<Span>,
    name: &'de str,
    values: &'de [TokenTree<'de>],
}

impl<'de> de::EnumAccess<'de> for EnumAccessor<'de> {
    type Error = DeserError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(IdentifierDeserializer { name: self.name })?;
        Ok((val, self))
    }
}

impl<'de> de::VariantAccess<'de> for EnumAccessor<'de> {
    type Error = DeserError;

    // `enum Enum { UnitVariant }`
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    // `enum Enum { NewtypeVariant(a) }`
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.values.len() != 1 {
            return Err(DeserError::new(
                self.name_span,
                format!(
                    "Expected exactly one value for enum variant `{}`",
                    self.name
                ),
            ));
        }

        seed.deserialize(TokenTreeDeserializer::new(&self.values[0]))
    }

    // `enum Enum { TupleVariant(a, b, c) }`
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if len != self.values.len() {
            if self.values.len() == 1 {
                if let Some(list) = self.values[0].as_list() {
                    if list.values.len() == len {
                        // Allow `"TupleVariant"([1, 2, 3])` to be interpreted as `"TupleVariant"(1, 2, 3)`
                        return visitor.visit_seq(ListAccessor(&list.values));
                    }
            }

            return Err(DeserError::new(
                self.name_span,
                format!(
                    "Expected {} values for enum variant `{}`, got {}",
                    len,
                    self.name,
                    self.values.len()
                ),
            ));
        }

        visitor.visit_seq(ListAccessor(self.values))
    }

    // `enum Enum { StructVariant{ a: … } }`
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.values.len() != 1 {
            return Err(DeserError::new(
                self.name_span,
                format!(
                    "Expected exactly one value for enum variant `{}`",
                    self.name
                ),
            ));
        }

        TokenTreeDeserializer::new(&self.values[0]).deserialize_any(visitor)
    }
}

struct IdentifierDeserializer<'de> {
    name: &'de str,
}

impl<'de> de::Deserializer<'de> for IdentifierDeserializer<'de> {
    type Error = DeserError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.name)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple enum option
        tuple_struct map struct identifier ignored_any
    }
}
