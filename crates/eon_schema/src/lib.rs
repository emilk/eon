//! Experimental schema model for Eon configuration.
//!
//! The schema model is intentionally independent from parsing and composition.
//! Tools can derive or construct schemas, then use them for completion,
//! validation, generated examples, and documentation.

use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

#[cfg(feature = "derive")]
pub use eon_schema_derive::EonSchema;

/// A Rust type that can describe its Eon schema.
pub trait EonSchema {
    /// Return the schema for this type.
    fn schema() -> SchemaNode;
}

/// Describes an Eon value shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaNode {
    /// Any Eon value.
    Any,
    /// The `null` value.
    Null,
    /// A boolean value.
    Bool,
    /// An integer value.
    Integer(IntegerSchema),
    /// A floating-point value.
    Float(FloatSchema),
    /// A number when integer-vs-float distinction is not known.
    Number,
    /// A string value.
    String(StringSchema),
    /// An optional value.
    Optional(Box<SchemaNode>),
    /// A list or sequence.
    List(Box<SchemaNode>),
    /// A map/table with arbitrary key and value schemas.
    Map {
        /// Schema for map keys.
        key: Box<SchemaNode>,
        /// Schema for map values.
        value: Box<SchemaNode>,
    },
    /// A record/object with named fields.
    Object(ObjectSchema),
    /// An enum/sum type.
    Enum(EnumSchema),
}

/// Schema for integer values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerSchema {
    /// Whether the integer may be negative.
    pub signed: bool,
    /// Number of bits in the Rust source type.
    pub bits: u16,
}

/// Schema for floating-point values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloatSchema {
    /// Number of bits in the Rust source type.
    pub bits: u16,
}

/// Schema for strings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StringSchema {
    /// Optional fixed choices for this string.
    pub choices: Vec<&'static str>,
}

/// Schema for an object/struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectSchema {
    /// Rust or renamed type name.
    pub name: &'static str,
    /// Documentation attached to the type.
    pub docs: &'static str,
    /// Field schemas in declaration order.
    pub fields: Vec<FieldSchema>,
    /// Whether fields outside `fields` are accepted.
    pub open: bool,
    /// Tool-specific extension metadata.
    pub extensions: Vec<SchemaExtension>,
}

/// Schema for a named object field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSchema {
    /// Eon field name.
    pub name: &'static str,
    /// Documentation attached to the field.
    pub docs: &'static str,
    /// Field value schema.
    pub ty: SchemaNode,
    /// Whether the field must be present.
    pub required: bool,
    /// Whether the Rust type/serde shape supplies a default.
    pub default: bool,
    /// Optional deprecation note.
    pub deprecated: Option<&'static str>,
    /// Tool-specific extension metadata.
    pub extensions: Vec<SchemaExtension>,
}

/// Schema for an enum/sum type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumSchema {
    /// Rust or renamed enum name.
    pub name: &'static str,
    /// Documentation attached to the enum.
    pub docs: &'static str,
    /// Variant schemas in declaration order.
    pub variants: Vec<VariantSchema>,
    /// Tool-specific extension metadata.
    pub extensions: Vec<SchemaExtension>,
}

/// Schema for one enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariantSchema {
    /// Eon variant name.
    pub name: &'static str,
    /// Documentation attached to the variant.
    pub docs: &'static str,
    /// Payload shape.
    pub payload: VariantPayload,
    /// Optional deprecation note.
    pub deprecated: Option<&'static str>,
    /// Tool-specific extension metadata.
    pub extensions: Vec<SchemaExtension>,
}

/// Payload shape for an enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariantPayload {
    /// Unit variant such as `Release`.
    Unit,
    /// Tuple payload such as `Rgb(255, 0, 0)`.
    Tuple(Vec<SchemaNode>),
    /// Struct payload such as `Rgb({ r: 255, g: 0, b: 0 })`.
    Struct(Vec<FieldSchema>),
}

/// Namespaced metadata for tools layered on top of Eon schemas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaExtension {
    /// Namespaced extension key, e.g. `vsr.command`.
    pub key: &'static str,
    /// Extension value.
    pub value: &'static str,
}

impl SchemaNode {
    /// Return `true` if this node accepts absent/null-like values.
    #[must_use]
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_) | Self::Null)
    }

    /// Parse a schema node from an Eon schema artifact string.
    ///
    /// This is intended for generated schema artifacts consumed by tools such
    /// as language servers. Strings are interned for the process lifetime so
    /// the result can share the same `'static` schema model as derived schemas.
    pub fn from_eon_str(source: &str) -> Result<Self, SchemaParseError> {
        let value = source
            .parse::<eon::Value>()
            .map_err(|err| SchemaParseError::new(err.to_string()))?;
        Self::from_eon_value(&value)
    }

    /// Parse a schema node from an Eon value.
    pub fn from_eon_value(value: &eon::Value) -> Result<Self, SchemaParseError> {
        parse_schema_node(value)
    }
}

/// Error returned while parsing an Eon schema artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaParseError {
    message: String,
}

impl SchemaParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SchemaParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for SchemaParseError {}

fn parse_schema_node(value: &eon::Value) -> Result<SchemaNode, SchemaParseError> {
    if let Some(kind) = value.as_string() {
        return parse_named_schema_kind(kind, None);
    }

    let map = value
        .as_map()
        .ok_or_else(|| SchemaParseError::new("schema node must be a string or map"))?;
    let kind = required_str(map, "kind")?;
    parse_named_schema_kind(kind, Some(map))
}

fn parse_named_schema_kind(
    kind: &str,
    map: Option<&eon::Map>,
) -> Result<SchemaNode, SchemaParseError> {
    match kind {
        "any" => Ok(SchemaNode::Any),
        "null" => Ok(SchemaNode::Null),
        "bool" => Ok(SchemaNode::Bool),
        "integer" => {
            let Some(map) = map else {
                return Ok(SchemaNode::Integer(IntegerSchema {
                    signed: true,
                    bits: 64,
                }));
            };
            Ok(SchemaNode::Integer(IntegerSchema {
                signed: optional_bool(map, "signed")?.unwrap_or(true),
                bits: optional_u16(map, "bits")?.unwrap_or(64),
            }))
        }
        "float" => {
            let Some(map) = map else {
                return Ok(SchemaNode::Float(FloatSchema { bits: 64 }));
            };
            Ok(SchemaNode::Float(FloatSchema {
                bits: optional_u16(map, "bits")?.unwrap_or(64),
            }))
        }
        "number" => Ok(SchemaNode::Number),
        "string" => Ok(SchemaNode::String(parse_string_schema(map)?)),
        "optional" => {
            let map = require_map_options(kind, map)?;
            let item = required_value(map, "item").or_else(|_| required_value(map, "value"))?;
            Ok(SchemaNode::Optional(Box::new(parse_schema_node(item)?)))
        }
        "list" => {
            let map = require_map_options(kind, map)?;
            let item = required_value(map, "item").or_else(|_| required_value(map, "value"))?;
            Ok(SchemaNode::List(Box::new(parse_schema_node(item)?)))
        }
        "map" => {
            let map = require_map_options(kind, map)?;
            Ok(SchemaNode::Map {
                key: Box::new(parse_schema_node(required_value(map, "key")?)?),
                value: Box::new(parse_schema_node(required_value(map, "value")?)?),
            })
        }
        "object" => Ok(SchemaNode::Object(parse_object_schema(
            require_map_options(kind, map)?,
        )?)),
        "enum" => Ok(SchemaNode::Enum(parse_enum_schema(require_map_options(
            kind, map,
        )?)?)),
        _ => Err(SchemaParseError::new(format!(
            "unknown schema kind `{kind}`"
        ))),
    }
}

fn parse_string_schema(map: Option<&eon::Map>) -> Result<StringSchema, SchemaParseError> {
    let Some(map) = map else {
        return Ok(StringSchema::default());
    };
    let choices = optional_list(map, "choices")?
        .unwrap_or(&[])
        .iter()
        .map(expect_string)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StringSchema { choices })
}

fn parse_object_schema(map: &eon::Map) -> Result<ObjectSchema, SchemaParseError> {
    let fields = optional_list(map, "fields")?
        .unwrap_or(&[])
        .iter()
        .map(parse_field_schema)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ObjectSchema {
        name: optional_static_str(map, "name")?.unwrap_or(""),
        docs: optional_static_str(map, "docs")?.unwrap_or(""),
        fields,
        open: optional_bool(map, "open")?.unwrap_or(false),
        extensions: parse_extensions(map)?,
    })
}

fn parse_field_schema(value: &eon::Value) -> Result<FieldSchema, SchemaParseError> {
    let map = value
        .as_map()
        .ok_or_else(|| SchemaParseError::new("field schema must be a map"))?;
    let ty = required_value(map, "type").or_else(|_| required_value(map, "ty"))?;
    Ok(FieldSchema {
        name: required_static_str(map, "name")?,
        docs: optional_static_str(map, "docs")?.unwrap_or(""),
        ty: parse_schema_node(ty)?,
        required: optional_bool(map, "required")?.unwrap_or(true),
        default: optional_bool(map, "default")?.unwrap_or(false),
        deprecated: optional_static_str(map, "deprecated")?,
        extensions: parse_extensions(map)?,
    })
}

fn parse_enum_schema(map: &eon::Map) -> Result<EnumSchema, SchemaParseError> {
    let variants = optional_list(map, "variants")?
        .unwrap_or(&[])
        .iter()
        .map(parse_variant_schema)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(EnumSchema {
        name: optional_static_str(map, "name")?.unwrap_or(""),
        docs: optional_static_str(map, "docs")?.unwrap_or(""),
        variants,
        extensions: parse_extensions(map)?,
    })
}

fn parse_variant_schema(value: &eon::Value) -> Result<VariantSchema, SchemaParseError> {
    let map = value
        .as_map()
        .ok_or_else(|| SchemaParseError::new("variant schema must be a map"))?;
    Ok(VariantSchema {
        name: required_static_str(map, "name")?,
        docs: optional_static_str(map, "docs")?.unwrap_or(""),
        payload: parse_variant_payload(map.get_str("payload"))?,
        deprecated: optional_static_str(map, "deprecated")?,
        extensions: parse_extensions(map)?,
    })
}

fn parse_variant_payload(value: Option<&eon::Value>) -> Result<VariantPayload, SchemaParseError> {
    let Some(value) = value else {
        return Ok(VariantPayload::Unit);
    };
    if let Some(kind) = value.as_string() {
        return match kind {
            "unit" => Ok(VariantPayload::Unit),
            _ => Err(SchemaParseError::new(format!(
                "unknown variant payload kind `{kind}`"
            ))),
        };
    }

    let map = value
        .as_map()
        .ok_or_else(|| SchemaParseError::new("variant payload must be a string or map"))?;
    let kind = required_str(map, "kind")?;
    match kind {
        "unit" => Ok(VariantPayload::Unit),
        "tuple" => {
            let values = optional_list(map, "values")?
                .unwrap_or(&[])
                .iter()
                .map(parse_schema_node)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(VariantPayload::Tuple(values))
        }
        "struct" => {
            let fields = optional_list(map, "fields")?
                .unwrap_or(&[])
                .iter()
                .map(parse_field_schema)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(VariantPayload::Struct(fields))
        }
        _ => Err(SchemaParseError::new(format!(
            "unknown variant payload kind `{kind}`"
        ))),
    }
}

fn parse_extensions(map: &eon::Map) -> Result<Vec<SchemaExtension>, SchemaParseError> {
    let Some(value) = map.get_str("extensions") else {
        return Ok(Vec::new());
    };
    let extensions = value
        .as_map()
        .ok_or_else(|| SchemaParseError::new("extensions must be a map"))?;
    extensions
        .into_iter()
        .map(|(key, value)| {
            let key = key
                .as_string()
                .ok_or_else(|| SchemaParseError::new("extension keys must be strings"))?;
            let value = value
                .as_string()
                .ok_or_else(|| SchemaParseError::new("extension values must be strings"))?;
            Ok(SchemaExtension {
                key: intern(key),
                value: intern(value),
            })
        })
        .collect()
}

fn require_map_options<'a>(
    kind: &str,
    map: Option<&'a eon::Map>,
) -> Result<&'a eon::Map, SchemaParseError> {
    map.ok_or_else(|| SchemaParseError::new(format!("`{kind}` schema must be a map")))
}

fn required_value<'a>(map: &'a eon::Map, key: &str) -> Result<&'a eon::Value, SchemaParseError> {
    map.get_str(key)
        .ok_or_else(|| SchemaParseError::new(format!("missing `{key}`")))
}

fn required_str<'a>(map: &'a eon::Map, key: &str) -> Result<&'a str, SchemaParseError> {
    required_value(map, key).and_then(expect_borrowed_string)
}

fn required_static_str(map: &eon::Map, key: &str) -> Result<&'static str, SchemaParseError> {
    required_str(map, key).map(intern)
}

fn optional_static_str(
    map: &eon::Map,
    key: &str,
) -> Result<Option<&'static str>, SchemaParseError> {
    map.get_str(key)
        .map(expect_borrowed_string)
        .transpose()
        .map(|value| value.map(intern))
}

fn optional_bool(map: &eon::Map, key: &str) -> Result<Option<bool>, SchemaParseError> {
    map.get_str(key)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| SchemaParseError::new(format!("`{key}` must be a bool")))
        })
        .transpose()
}

fn optional_u16(map: &eon::Map, key: &str) -> Result<Option<u16>, SchemaParseError> {
    map.get_str(key)
        .map(|value| {
            let number = value
                .as_number()
                .ok_or_else(|| SchemaParseError::new(format!("`{key}` must be a number")))?;
            let Some(value) = number.as_u64() else {
                return Err(SchemaParseError::new(format!(
                    "`{key}` must be an unsigned integer"
                )));
            };
            u16::try_from(value)
                .map_err(|_err| SchemaParseError::new(format!("`{key}` is out of range for u16")))
        })
        .transpose()
}

fn optional_list<'a>(
    map: &'a eon::Map,
    key: &str,
) -> Result<Option<&'a [eon::Value]>, SchemaParseError> {
    map.get_str(key)
        .map(|value| {
            value
                .as_list()
                .ok_or_else(|| SchemaParseError::new(format!("`{key}` must be a list")))
        })
        .transpose()
}

fn expect_string(value: &eon::Value) -> Result<&'static str, SchemaParseError> {
    expect_borrowed_string(value).map(intern)
}

fn expect_borrowed_string(value: &eon::Value) -> Result<&str, SchemaParseError> {
    value
        .as_string()
        .ok_or_else(|| SchemaParseError::new("expected string"))
}

fn intern(value: &str) -> &'static str {
    Box::leak(value.to_owned().into_boxed_str())
}

impl EonSchema for () {
    fn schema() -> SchemaNode {
        SchemaNode::Null
    }
}

impl EonSchema for bool {
    fn schema() -> SchemaNode {
        SchemaNode::Bool
    }
}

impl EonSchema for String {
    fn schema() -> SchemaNode {
        SchemaNode::String(StringSchema::default())
    }
}

impl EonSchema for str {
    fn schema() -> SchemaNode {
        SchemaNode::String(StringSchema::default())
    }
}

impl<T> EonSchema for Option<T>
where
    T: EonSchema,
{
    fn schema() -> SchemaNode {
        SchemaNode::Optional(Box::new(T::schema()))
    }
}

impl<T> EonSchema for Vec<T>
where
    T: EonSchema,
{
    fn schema() -> SchemaNode {
        SchemaNode::List(Box::new(T::schema()))
    }
}

impl<T, const N: usize> EonSchema for [T; N]
where
    T: EonSchema,
{
    fn schema() -> SchemaNode {
        SchemaNode::List(Box::new(T::schema()))
    }
}

impl<T> EonSchema for Box<T>
where
    T: EonSchema,
{
    fn schema() -> SchemaNode {
        T::schema()
    }
}

impl<K, V> EonSchema for BTreeMap<K, V>
where
    K: EonSchema,
    V: EonSchema,
{
    fn schema() -> SchemaNode {
        SchemaNode::Map {
            key: Box::new(K::schema()),
            value: Box::new(V::schema()),
        }
    }
}

impl<K, V> EonSchema for HashMap<K, V>
where
    K: EonSchema,
    V: EonSchema,
{
    fn schema() -> SchemaNode {
        SchemaNode::Map {
            key: Box::new(K::schema()),
            value: Box::new(V::schema()),
        }
    }
}

macro_rules! impl_integer_schema {
    ($($ty:ty => ($signed:literal, $bits:expr)),* $(,)?) => {
        $(
            impl EonSchema for $ty {
                fn schema() -> SchemaNode {
                    SchemaNode::Integer(IntegerSchema {
                        signed: $signed,
                        bits: $bits,
                    })
                }
            }
        )*
    };
}

impl_integer_schema! {
    i8 => (true, 8),
    i16 => (true, 16),
    i32 => (true, 32),
    i64 => (true, 64),
    i128 => (true, 128),
    isize => (true, usize::BITS as u16),
    u8 => (false, 8),
    u16 => (false, 16),
    u32 => (false, 32),
    u64 => (false, 64),
    u128 => (false, 128),
    usize => (false, usize::BITS as u16),
}

macro_rules! impl_float_schema {
    ($($ty:ty => $bits:literal),* $(,)?) => {
        $(
            impl EonSchema for $ty {
                fn schema() -> SchemaNode {
                    SchemaNode::Float(FloatSchema { bits: $bits })
                }
            }
        )*
    };
}

impl_float_schema! {
    f32 => 32,
    f64 => 64,
}

#[cfg(test)]
mod tests {
    use super::{EonSchema as _, SchemaNode, VariantPayload};

    #[test]
    fn option_schema_is_optional() {
        let schema = Option::<String>::schema();
        assert!(schema.is_optional());
    }

    #[test]
    fn map_schema_has_key_and_value_shapes() {
        let schema = std::collections::BTreeMap::<String, u32>::schema();
        let SchemaNode::Map { key, value } = schema else {
            panic!("expected map schema");
        };

        assert!(matches!(*key, SchemaNode::String(_)));
        assert!(matches!(*value, SchemaNode::Integer(_)));
    }

    #[test]
    fn parses_object_schema_artifact() {
        let schema = SchemaNode::from_eon_str(
            r#"
kind: "object"
name: "Config"
docs: "Application config."
fields: [
    {
        name: "port"
        docs: "Server port."
        type: { kind: "integer", signed: false, bits: 16 }
        required: true
    }
    {
        name: "tags"
        type: { kind: "list", item: "string" }
        required: false
    }
]
extensions: {
    "vsr.command": "run"
}
"#,
        )
        .unwrap();

        let SchemaNode::Object(object) = schema else {
            panic!("expected object schema");
        };
        assert_eq!(object.name, "Config");
        assert_eq!(object.docs, "Application config.");
        assert_eq!(object.fields.len(), 2);
        assert_eq!(object.fields[0].name, "port");
        assert!(object.fields[0].required);
        assert_eq!(object.extensions[0].key, "vsr.command");
    }

    #[test]
    fn parses_enum_schema_artifact() {
        let schema = SchemaNode::from_eon_str(
            r#"
kind: "enum"
name: "Color"
variants: [
    { name: "Black" }
    {
        name: "Rgb"
        payload: {
            kind: "tuple"
            values: ["integer", "integer", "integer"]
        }
    }
]
"#,
        )
        .unwrap();

        let SchemaNode::Enum(schema) = schema else {
            panic!("expected enum schema");
        };
        assert_eq!(schema.variants[0].payload, VariantPayload::Unit);
        let VariantPayload::Tuple(values) = &schema.variants[1].payload else {
            panic!("expected tuple payload");
        };
        assert_eq!(values.len(), 3);
    }
}
