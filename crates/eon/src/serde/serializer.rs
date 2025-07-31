// See https://serde.rs/impl-serializer.html

use serde::{
    Serialize,
    ser::{self, Error as _},
};

use crate::{Map, Value, serde::to_value, value::Variant};

use vec1::vec1;

/// An error during serde-serialization into Eon.
#[derive(Debug, Clone)]
pub struct SerializationError {
    msg: String,
}

impl std::error::Error for SerializationError {}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

impl ser::Error for SerializationError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

type Result<T = (), E = SerializationError> = std::result::Result<T, E>;

#[derive(Default)]
pub struct Serializer {}

impl ser::Serializer for &'_ Serializer {
    // What we produce as output.
    type Ok = Value;

    type Error = SerializationError;

    type SerializeSeq = ListSerializer;
    type SerializeTuple = ListSerializer;
    type SerializeTupleStruct = ListSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::from(v))
    }

    // Serialize a char as a single-character string.
    #[inline]
    fn serialize_char(self, v: char) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::from(v.to_owned()))
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        Ok(Value::from(v))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value> {
        Ok(Value::Null)
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`.
    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // ()
    #[inline]
    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    // Unit struct means a named value containing no data.
    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
    ) -> Result<Value> {
        Ok(Value::new_variant(variant_name.to_owned(), vec![]))
    }

    // Treat newtype structs as insignificant wrappers around the data they contain.
    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // `enum Enum { VariantName(Value), … }`
    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        Ok(Value::Variant(Variant {
            name: variant_name.to_owned(),
            values: vec1![value.serialize(self)?],
        }))
    }

    /// Serialize a list
    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(ListSerializer::with_capacity(len.unwrap_or(0)))
    }

    /// Serialize tuples as lists, so (a b c) is the same as [a b c].
    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(ListSerializer::with_capacity(len))
    }

    /// Named tuples, e.g. `struct Rgb(u8, u8, u8)`.
    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(ListSerializer::with_capacity(len))
    }

    /// Enum variant that are tuples, e.g. `enum Color { Rgb(u8, u8, u8), … }`.
    #[inline]
    fn serialize_tuple_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleVariantSerializer::with_capacity(variant_name, len))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer::with_capacity(len.unwrap_or(0)))
    }

    #[inline]
    fn serialize_struct(
        self,
        _struct_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(MapSerializer::with_capacity(len))
    }

    /// ```ignore
    /// enum EnumName {
    ///     VariantName {
    ///         key: Value,
    ///         …
    ///     },
    ///     …
    /// }
    /// ```
    #[inline]
    fn serialize_struct_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructVariantSerializer::with_capacity(variant_name, len))
    }
}

// -----------------------------------------------------------------------------------------------

/// [a, b, c]
pub struct ListSerializer {
    list: Vec<Value>,
}

impl ListSerializer {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            list: Vec::with_capacity(capacity),
        }
    }
}

impl ser::SerializeSeq for ListSerializer {
    type Ok = Value;
    type Error = SerializationError;

    // Serialize a single element of the sequence.
    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.list.push(to_value(value)?);
        Ok(())
    }

    // Close the sequence.
    #[inline]
    fn end(self) -> Result<Value> {
        Ok(Value::List(self.list))
    }
}

/// (a, b, c)
impl ser::SerializeTuple for ListSerializer {
    type Ok = Value;
    type Error = SerializationError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.list.push(to_value(value)?);
        Ok(())
    }

    // Close the sequence.
    #[inline]
    fn end(self) -> Result<Value> {
        Ok(Value::List(self.list))
    }
}
// Named tuples, e.g. `struct Rgb(u8, u8, u8)`.
impl ser::SerializeTupleStruct for ListSerializer {
    type Ok = Value;
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.list.push(to_value(value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Value> {
        Ok(Value::List(self.list))
    }
}

// -----------------------------------------------------------------------------------------------

/// Enum variant that are tuples, e.g. `enum Color { Rgb(u8, u8, u8), … }`.
pub struct TupleVariantSerializer {
    variant_name: &'static str,
    values: Vec<Value>,
}

impl TupleVariantSerializer {
    fn with_capacity(variant_name: &'static str, capacity: usize) -> Self {
        Self {
            variant_name,
            values: Vec::with_capacity(capacity),
        }
    }
}

impl ser::SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(to_value(value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Value> {
        let Self {
            variant_name,
            values,
        } = self;
        Ok(Value::new_variant(variant_name.to_owned(), values))
    }
}

// -----------------------------------------------------------------------------------------------

/// Used for maps, structs, and enum variants that are structs.
pub struct MapSerializer {
    map: Map,
    last_key: Option<Value>,
}

impl MapSerializer {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            map: Map::with_capacity(capacity),
            last_key: None,
        }
    }
}

impl ser::SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = SerializationError;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.last_key.is_some() {
            return Err(SerializationError::custom(
                "serialize_key called twice without serialize_value",
            ));
        }

        self.last_key = Some(to_value(key)?);

        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if let Some(key) = self.last_key.take() {
            self.map.insert(key, to_value(value)?);
            Ok(())
        } else {
            Err(SerializationError::custom(
                "serialize_value called without serialize_key",
            ))
        }
    }

    #[inline]
    fn end(self) -> Result<Value> {
        if self.last_key.is_some() {
            return Err(SerializationError::custom(
                "serialize_value not called after serialize_key",
            ));
        }
        Ok(Value::Map(self.map))
    }
}

impl ser::SerializeStruct for MapSerializer {
    type Ok = Value;
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(to_value(key)?, to_value(value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Value> {
        Ok(Value::Map(self.map))
    }
}

// -----------------------------------------------------------------------------------------------

/// For enum variants containing a struct, e.g. `enum EnumName { VariantName { key: Value, … }, … }`.
pub struct StructVariantSerializer {
    name: &'static str,
    map: Map,
}

impl StructVariantSerializer {
    fn with_capacity(name: &'static str, capacity: usize) -> Self {
        Self {
            name,
            map: Map::with_capacity(capacity),
        }
    }
}

/// ```ignore
/// enum EnumName {
///     VariantName {
///         key: Value,
///         …
///     },
///     …
/// }
/// ```
impl ser::SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(to_value(key)?, to_value(value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Value> {
        Ok(Value::Variant(Variant {
            name: self.name.to_owned(),
            values: vec1![Value::Map(self.map)],
        }))
    }
}
