use std::collections::HashSet;
use std::str::FromStr as _;

use eon_core::{
    Event, EventSink, ParseError, Scalar, Span as CoreSpan, SpannedEvent, StringToken, VariantName,
    parse,
};
use eon_syntax::{Span, unescape_and_unquote};
use serde::{
    Deserializer as _,
    de::{self, Error as _, Visitor},
};

use crate::{Map, Number, Value};

use super::deserializer::DeserError;

type Result<T = (), E = DeserError> = std::result::Result<T, E>;

/// Parse an Eon value from a string into a type `T` using the experimental
/// `eon_core` parser and a lightweight borrowed core tree.
pub fn from_str_with_core<T>(eon_source: &str) -> std::result::Result<T, crate::Error>
where
    T: serde::de::DeserializeOwned,
{
    let mut collector = TreeCollector::default();
    match parse(eon_source, &mut collector) {
        Ok(()) => {
            let root = collector
                .finish()
                .map_err(|err| err.into_error(eon_source))?;
            T::deserialize(NodeDeserializer::new(&root)).map_err(|err| err.into_error(eon_source))
        }
        Err(ParseError::Parse(err)) => Err(core_error_to_eon_error(eon_source, err)),
        Err(ParseError::Sink(err)) => Err(err.into_error(eon_source)),
    }
}

#[derive(Debug)]
struct Node<'a> {
    span: Option<Span>,
    value: NodeValue<'a>,
}

#[derive(Debug)]
enum NodeValue<'a> {
    Null,
    Bool(bool),
    Number(&'a str),
    Identifier(&'a str),
    String(StringToken<'a>),
    List(Vec<Node<'a>>),
    Map(Vec<KeyValue<'a>>),
    Variant(VariantNode<'a>),
}

#[derive(Debug)]
struct KeyValue<'a> {
    key: Node<'a>,
    value: Node<'a>,
}

#[derive(Debug)]
struct VariantNode<'a> {
    name: VariantName<'a>,
    values: Vec<Node<'a>>,
}

#[derive(Debug)]
enum Frame<'a> {
    Map {
        start: Span,
        entries: Vec<KeyValue<'a>>,
        pending_key: Option<Node<'a>>,
        phase: MapPhase,
    },
    List {
        start: Span,
        values: Vec<Node<'a>>,
    },
    Variant {
        start: Span,
        name: VariantName<'a>,
        values: Vec<Node<'a>>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapPhase {
    ExpectKeyMarker,
    WritingKey,
    ExpectValueMarker,
    WritingValue,
}

#[derive(Default)]
struct TreeCollector<'a> {
    stack: Vec<Frame<'a>>,
    root: Option<Node<'a>>,
}

#[derive(Debug)]
enum CollectErrorKind {
    InvalidState(&'static str),
    MissingRoot,
}

#[derive(Debug)]
struct CollectError {
    span: Option<Span>,
    kind: CollectErrorKind,
}

impl CollectError {
    fn at(span: Span, kind: CollectErrorKind) -> Self {
        Self {
            span: Some(span),
            kind,
        }
    }

    fn custom(kind: CollectErrorKind) -> Self {
        Self { span: None, kind }
    }

    fn into_error(self, eon_source: &str) -> crate::Error {
        let message = match self.kind {
            CollectErrorKind::InvalidState(msg) => msg.to_owned(),
            CollectErrorKind::MissingRoot => {
                "Expected the parser to produce a root value".to_owned()
            }
        };

        crate::Error::new(eon_source, self.span, message)
    }
}

impl<'a> TreeCollector<'a> {
    fn finish(self) -> std::result::Result<Node<'a>, CollectError> {
        if !self.stack.is_empty() {
            return Err(CollectError::custom(CollectErrorKind::InvalidState(
                "Parser finished with unterminated containers",
            )));
        }

        self.root
            .ok_or_else(|| CollectError::custom(CollectErrorKind::MissingRoot))
    }

    fn push_node(&mut self, node: Node<'a>) -> std::result::Result<(), CollectError> {
        let Some(frame) = self.stack.last_mut() else {
            if self.root.replace(node).is_some() {
                return Err(CollectError::custom(CollectErrorKind::InvalidState(
                    "Parser emitted multiple root values",
                )));
            }
            return Ok(());
        };

        match frame {
            Frame::List { values, .. } => {
                values.push(node);
                Ok(())
            }
            Frame::Variant { values, .. } => {
                values.push(node);
                Ok(())
            }
            Frame::Map {
                entries,
                pending_key,
                phase,
                ..
            } => match phase {
                MapPhase::WritingKey => {
                    *pending_key = Some(node);
                    *phase = MapPhase::ExpectValueMarker;
                    Ok(())
                }
                MapPhase::WritingValue => {
                    let Some(key) = pending_key.take() else {
                        return Err(CollectError::at(
                            node.span.unwrap_or(Span { start: 0, end: 0 }),
                            CollectErrorKind::InvalidState("Missing map key before map value"),
                        ));
                    };
                    entries.push(KeyValue { key, value: node });
                    *phase = MapPhase::ExpectKeyMarker;
                    Ok(())
                }
                MapPhase::ExpectKeyMarker => Err(CollectError::at(
                    node.span.unwrap_or(Span { start: 0, end: 0 }),
                    CollectErrorKind::InvalidState(
                        "Map received a value without a preceding key marker",
                    ),
                )),
                MapPhase::ExpectValueMarker => Err(CollectError::at(
                    node.span.unwrap_or(Span { start: 0, end: 0 }),
                    CollectErrorKind::InvalidState(
                        "Map received a value without a preceding value marker",
                    ),
                )),
            },
        }
    }
}

impl<'a> EventSink<'a> for TreeCollector<'a> {
    type Error = CollectError;

    fn event(&mut self, event: SpannedEvent<'a>) -> std::result::Result<(), Self::Error> {
        let span = core_span_to_syntax_span(event.span);

        match event.event {
            Event::BeginMap { .. } => {
                self.stack.push(Frame::Map {
                    start: span,
                    entries: Vec::new(),
                    pending_key: None,
                    phase: MapPhase::ExpectKeyMarker,
                });
                Ok(())
            }
            Event::EndMap => {
                let Some(Frame::Map {
                    start,
                    entries,
                    phase,
                    ..
                }) = self.stack.pop()
                else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("EndMap without BeginMap"),
                    ));
                };

                if phase != MapPhase::ExpectKeyMarker {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState(
                            "Map ended while a key or value was incomplete",
                        ),
                    ));
                }

                self.push_node(Node {
                    span: Some(combine_span(start, span)),
                    value: NodeValue::Map(entries),
                })
            }
            Event::MapKey => {
                let Some(Frame::Map { phase, .. }) = self.stack.last_mut() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("MapKey outside of a map"),
                    ));
                };

                if *phase != MapPhase::ExpectKeyMarker {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("Unexpected MapKey marker"),
                    ));
                }

                *phase = MapPhase::WritingKey;
                Ok(())
            }
            Event::MapValue => {
                let Some(Frame::Map { phase, .. }) = self.stack.last_mut() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("MapValue outside of a map"),
                    ));
                };

                if *phase != MapPhase::ExpectValueMarker {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("Unexpected MapValue marker"),
                    ));
                }

                *phase = MapPhase::WritingValue;
                Ok(())
            }
            Event::BeginList => {
                self.stack.push(Frame::List {
                    start: span,
                    values: Vec::new(),
                });
                Ok(())
            }
            Event::EndList => {
                let Some(Frame::List { start, values }) = self.stack.pop() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("EndList without BeginList"),
                    ));
                };

                self.push_node(Node {
                    span: Some(combine_span(start, span)),
                    value: NodeValue::List(values),
                })
            }
            Event::BeginVariant { name } => {
                self.stack.push(Frame::Variant {
                    start: span,
                    name,
                    values: Vec::new(),
                });
                Ok(())
            }
            Event::EndVariant => {
                let Some(Frame::Variant {
                    start,
                    name,
                    values,
                }) = self.stack.pop()
                else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("EndVariant without BeginVariant"),
                    ));
                };

                self.push_node(Node {
                    span: Some(combine_span(start, span)),
                    value: NodeValue::Variant(VariantNode { name, values }),
                })
            }
            Event::Scalar(scalar) => self.push_node(Node {
                span: Some(span),
                value: match scalar {
                    Scalar::Null => NodeValue::Null,
                    Scalar::Bool(value) => NodeValue::Bool(value),
                    Scalar::Number(number) => NodeValue::Number(number),
                    Scalar::Identifier(identifier) => NodeValue::Identifier(identifier),
                    Scalar::String(string) => NodeValue::String(string),
                },
            }),
        }
    }
}

struct NodeDeserializer<'tree, 'de> {
    value: &'tree Node<'de>,
}

impl<'tree, 'de> NodeDeserializer<'tree, 'de> {
    fn new(value: &'tree Node<'de>) -> Self {
        Self { value }
    }
}

impl<'tree, 'de> de::Deserializer<'de> for NodeDeserializer<'tree, 'de> {
    type Error = DeserError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let span = self.value.span;

        let mut result = match &self.value.value {
            NodeValue::Null => visitor.visit_unit(),
            NodeValue::Bool(value) => visitor.visit_bool(*value),
            NodeValue::Identifier(identifier) => visitor.visit_borrowed_str(identifier),
            NodeValue::Number(num_str) => visit_number(span, num_str, visitor),
            NodeValue::String(StringToken { raw, .. }) => unescape_and_unquote(raw)
                .map_err(|err| {
                    DeserError::new(
                        span,
                        format!("Failed to unescape quoted string: {raw:?}: {err}"),
                    )
                })
                .and_then(|unescaped| visitor.visit_string(unescaped)),
            NodeValue::List(list) => visitor.visit_seq(ListAccessor(list)),
            NodeValue::Map(map) => visitor.visit_map(MapAccessor {
                kvs: map,
                seen_keys: HashSet::with_capacity(map.len()),
            }),
            NodeValue::Variant(_) => Err(DeserError::new(span, "Did not expect a variant here")),
        };

        if let Err(err) = &mut result {
            err.span = err.span.or(span);
        }

        result
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if matches!(self.value.value, NodeValue::Null) {
            return visitor.visit_none();
        }

        visitor.visit_some(self)
    }

    fn deserialize_enum<V>(
        self,
        _enum_name: &'static str,
        variant_names: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let values: &'tree [Node<'de>];
        let variant_name = match &self.value.value {
            NodeValue::Identifier(identifier) => {
                values = &[][..];
                *identifier
            }
            NodeValue::String(StringToken { raw, .. }) => {
                values = &[][..];
                let unescaped = unescape_and_unquote(raw).map_err(|err| {
                    DeserError::new(
                        self.value.span,
                        format!("Failed to unescape quoted name: {raw:?}: {err}"),
                    )
                })?;
                let Some(name) = variant_names.iter().find(|&&name| name == unescaped) else {
                    return Err(DeserError::new(
                        self.value.span,
                        format!("Expected one of: {variant_names:?}, got: {raw}"),
                    ));
                };
                return visitor.visit_enum(EnumAccessor {
                    name_span: self.value.span,
                    name,
                    values,
                });
            }
            NodeValue::Variant(variant) => {
                values = variant.values.as_slice();

                match variant.name {
                    VariantName::Identifier(identifier) => identifier,
                    VariantName::String(StringToken { raw, .. }) => {
                        let unescaped = unescape_and_unquote(raw).map_err(|err| {
                            DeserError::new(
                                self.value.span,
                                format!("Failed to unescape quoted name: {raw:?}: {err}"),
                            )
                        })?;

                        let Some(name) = variant_names.iter().find(|&&name| name == unescaped)
                        else {
                            return Err(DeserError::new(
                                self.value.span,
                                format!("Expected one of: {variant_names:?}, got: {raw}"),
                            ));
                        };

                        return visitor.visit_enum(EnumAccessor {
                            name_span: self.value.span,
                            name,
                            values,
                        });
                    }
                }
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
        };

        let Some(name) = variant_names.iter().find(|&&name| name == variant_name) else {
            return Err(DeserError::new(
                self.value.span,
                format!("Expected one of: {variant_names:?}, got: {variant_name}"),
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

fn visit_number<'de, V>(span: Option<Span>, num_str: &str, visitor: V) -> Result<V::Value>
where
    V: Visitor<'de>,
{
    match Number::from_str(num_str) {
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
                Err(DeserError::new(span, format!("Invalid number: {number}")))
            }
        }
        Err(err) => Err(DeserError::new(span, err)),
    }
}

struct ListAccessor<'tree, 'de>(&'tree [Node<'de>]);

impl<'tree, 'de> de::SeqAccess<'de> for ListAccessor<'tree, 'de> {
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
            seed.deserialize(NodeDeserializer::new(first)).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct MapAccessor<'tree, 'de> {
    kvs: &'tree [KeyValue<'de>],
    seen_keys: HashSet<Value>,
}

impl<'tree, 'de> de::MapAccess<'de> for MapAccessor<'tree, 'de> {
    type Error = DeserError;

    fn size_hint(&self) -> Option<usize> {
        Some(self.kvs.len())
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(kv) = self.kvs.first() {
            let key = key_identity_from_node(&kv.key)?;
            if !self.seen_keys.insert(key) {
                return Err(DeserError::new(kv.key.span, "Duplicate key in map"));
            }
            seed.deserialize(NodeDeserializer::new(&kv.key)).map(Some)
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
            seed.deserialize(NodeDeserializer::new(&first.value))
        } else {
            Err(DeserError::custom("No more values in map"))
        }
    }
}

struct EnumAccessor<'tree, 'de> {
    name_span: Option<Span>,
    name: &'de str,
    values: &'tree [Node<'de>],
}

impl<'tree, 'de> de::EnumAccess<'de> for EnumAccessor<'tree, 'de> {
    type Error = DeserError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(IdentifierDeserializer { name: self.name })?;
        Ok((value, self))
    }
}

impl<'tree, 'de> de::VariantAccess<'de> for EnumAccessor<'tree, 'de> {
    type Error = DeserError;

    fn unit_variant(self) -> Result<()> {
        if self.values.is_empty() {
            Ok(())
        } else {
            Err(DeserError::new(
                self.name_span,
                format!(
                    "Expected unit enum variant `{}` to have no payload values",
                    self.name
                ),
            ))
        }
    }

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

        seed.deserialize(NodeDeserializer::new(&self.values[0]))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if len != self.values.len() {
            if self.values.len() == 1 {
                if let NodeValue::List(list) = &self.values[0].value {
                    if list.len() == len {
                        return visitor.visit_seq(ListAccessor(list));
                    }
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

        NodeDeserializer::new(&self.values[0]).deserialize_any(visitor)
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

fn core_error_to_eon_error(eon_source: &str, error: eon_core::Error) -> crate::Error {
    crate::Error::new_at(
        eon_source,
        core_span_to_syntax_span(error.span),
        error.kind.to_string(),
    )
}

fn core_span_to_syntax_span(span: CoreSpan) -> Span {
    Span {
        start: span.start,
        end: span.end,
    }
}

fn combine_span(start: Span, end: Span) -> Span {
    Span {
        start: start.start,
        end: end.end,
    }
}

#[derive(Clone, Copy)]
enum ValuePosition {
    MapKey,
    Value,
}

fn key_identity_from_node(node: &Node<'_>) -> Result<Value> {
    node_to_value(node, ValuePosition::MapKey)
}

fn node_to_value(node: &Node<'_>, position: ValuePosition) -> Result<Value> {
    match &node.value {
        NodeValue::Null => match position {
            ValuePosition::MapKey => Ok(Value::String("null".to_owned())),
            ValuePosition::Value => Ok(Value::Null),
        },
        NodeValue::Bool(value) => match position {
            ValuePosition::MapKey => Ok(Value::String(value.to_string())),
            ValuePosition::Value => Ok(Value::Bool(*value)),
        },
        NodeValue::Identifier(identifier) => match position {
            ValuePosition::MapKey => Ok(Value::String((*identifier).to_owned())),
            ValuePosition::Value => Ok(Value::new_variant((*identifier).to_owned(), vec![])),
        },
        NodeValue::Number(raw) => Number::from_str(raw)
            .map(Value::Number)
            .map_err(|err| DeserError::new(node.span, err)),
        NodeValue::String(StringToken { raw, .. }) => {
            unescape_and_unquote(raw).map(Value::String).map_err(|err| {
                DeserError::new(
                    node.span,
                    format!("Failed to unescape quoted string: {raw:?}: {err}"),
                )
            })
        }
        NodeValue::List(list) => list
            .iter()
            .map(|value| node_to_value(value, ValuePosition::Value))
            .collect::<Result<Vec<_>>>()
            .map(Value::List),
        NodeValue::Map(map) => {
            let mut out = Map::with_capacity(map.len());
            for kv in map {
                let key = node_to_value(&kv.key, ValuePosition::MapKey)?;
                let value = node_to_value(&kv.value, ValuePosition::Value)?;
                if out.insert(key, value).is_some() {
                    return Err(DeserError::new(kv.key.span, "Duplicate key in map"));
                }
            }
            Ok(Value::Map(out))
        }
        NodeValue::Variant(variant) => {
            let name = match variant.name {
                VariantName::Identifier(identifier) => identifier.to_owned(),
                VariantName::String(StringToken { raw, .. }) => {
                    unescape_and_unquote(raw).map_err(|err| {
                        DeserError::new(
                            node.span,
                            format!("Failed to unescape quoted name: {raw:?}: {err}"),
                        )
                    })?
                }
            };

            let values = variant
                .values
                .iter()
                .map(|value| node_to_value(value, ValuePosition::Value))
                .collect::<Result<Vec<_>>>()?;

            Ok(Value::new_variant(name, values))
        }
    }
}
