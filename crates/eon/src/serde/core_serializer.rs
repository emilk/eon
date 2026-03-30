use std::fmt;

use eon_core::{is_valid_identifier, write_escaped_string, write_symbol};
use serde::{
    Serialize,
    ser::{self, Error as _, SerializeSeq as _},
};

use super::SerializationError;

type Result<T = (), E = SerializationError> = std::result::Result<T, E>;

macro_rules! serialize_integer {
    ($name:ident, $ty:ty) => {
        #[inline]
        fn $name(self, v: $ty) -> Result {
            write!(self.writer, "{v}").map_err(fmt_error)
        }
    };
}

/// Serialize a value (using serde) directly into the experimental compact
/// `eon_core` syntax without first constructing an owned [`crate::Value`].
pub fn to_string_with_core<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    serialize_fragment(value, Position::RootValue)
}

#[derive(Clone, Copy)]
enum Position {
    RootValue,
    MapKey,
    MapValue,
}

struct Serializer<'a, W> {
    writer: &'a mut W,
    position: Position,
}

impl<'a, W> ser::Serializer for Serializer<'a, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    type SerializeSeq = ListSerializer<'a, W>;
    type SerializeTuple = ListSerializer<'a, W>;
    type SerializeTupleStruct = ListSerializer<'a, W>;
    type SerializeTupleVariant = TupleVariantSerializer<'a, W>;
    type SerializeMap = MapSerializer<'a, W>;
    type SerializeStruct = MapSerializer<'a, W>;
    type SerializeStructVariant = StructVariantSerializer<'a, W>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result {
        if v {
            self.writer.write_str("true")
        } else {
            self.writer.write_str("false")
        }
        .map_err(fmt_error)
    }

    serialize_integer!(serialize_i8, i8);
    serialize_integer!(serialize_i16, i16);
    serialize_integer!(serialize_i32, i32);
    serialize_integer!(serialize_i64, i64);
    serialize_integer!(serialize_i128, i128);
    serialize_integer!(serialize_u8, u8);
    serialize_integer!(serialize_u16, u16);
    serialize_integer!(serialize_u32, u32);
    serialize_integer!(serialize_u64, u64);
    serialize_integer!(serialize_u128, u128);

    #[inline]
    fn serialize_f32(self, v: f32) -> Result {
        write_f32(self.writer, v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result {
        write_f64(self.writer, v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result {
        let mut buffer = [0_u8; 4];
        write_string_or_key(self.writer, v.encode_utf8(&mut buffer), self.position)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result {
        write_string_or_key(self.writer, v, self.position)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    #[inline]
    fn serialize_none(self) -> Result {
        self.writer.write_str("null").map_err(fmt_error)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result {
        self.writer.write_str("null").map_err(fmt_error)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result {
        self.writer.write_str("null").map_err(fmt_error)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
    ) -> Result {
        write_unit_variant(self.writer, variant_name, self.position)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        value: &T,
    ) -> Result
    where
        T: ?Sized + Serialize,
    {
        write_symbol(self.writer, variant_name).map_err(fmt_error)?;
        self.writer.write_char('(').map_err(fmt_error)?;
        value.serialize(Serializer {
            writer: self.writer,
            position: Position::MapValue,
        })?;
        self.writer.write_char(')').map_err(fmt_error)
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.writer.write_char('[').map_err(fmt_error)?;
        Ok(ListSerializer {
            writer: self.writer,
            first: true,
        })
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(None)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(None)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        write_symbol(self.writer, variant_name).map_err(fmt_error)?;
        self.writer.write_char('(').map_err(fmt_error)?;
        Ok(TupleVariantSerializer {
            writer: self.writer,
            first: true,
        })
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        MapSerializer::begin(self.writer, self.position, MapKind::Dynamic)
    }

    #[inline]
    fn serialize_struct(
        self,
        _struct_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        MapSerializer::begin(self.writer, self.position, MapKind::Struct)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        write_symbol(self.writer, variant_name).map_err(fmt_error)?;
        self.writer.write_char('(').map_err(fmt_error)?;
        let map = MapSerializer::begin(self.writer, Position::MapValue, MapKind::Struct)?;
        Ok(StructVariantSerializer { map })
    }
}

struct ListSerializer<'a, W> {
    writer: &'a mut W,
    first: bool,
}

impl<W> ser::SerializeSeq for ListSerializer<'_, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        if !self.first {
            self.writer.write_str(", ").map_err(fmt_error)?;
        }
        self.first = false;
        value.serialize(Serializer {
            writer: self.writer,
            position: Position::MapValue,
        })
    }

    #[inline]
    fn end(self) -> Result {
        self.writer.write_char(']').map_err(fmt_error)
    }
}

impl<W> ser::SerializeTuple for ListSerializer<'_, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result {
        ser::SerializeSeq::end(self)
    }
}

impl<W> ser::SerializeTupleStruct for ListSerializer<'_, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result {
        ser::SerializeSeq::end(self)
    }
}

struct TupleVariantSerializer<'a, W> {
    writer: &'a mut W,
    first: bool,
}

impl<W> ser::SerializeTupleVariant for TupleVariantSerializer<'_, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        if !self.first {
            self.writer.write_str(", ").map_err(fmt_error)?;
        }
        self.first = false;
        value.serialize(Serializer {
            writer: self.writer,
            position: Position::MapValue,
        })
    }

    #[inline]
    fn end(self) -> Result {
        self.writer.write_char(')').map_err(fmt_error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MapExpectation {
    Key,
    Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MapMode {
    Explicit,
    ImplicitRoot,
    PendingRootMap,
}

#[derive(Clone, Copy)]
enum MapKind {
    Dynamic,
    Struct,
}

struct MapSerializer<'a, W> {
    writer: &'a mut W,
    expectation: MapExpectation,
    first: bool,
    mode: MapMode,
}

impl<'a, W> MapSerializer<'a, W>
where
    W: fmt::Write,
{
    fn begin(writer: &'a mut W, position: Position, kind: MapKind) -> Result<Self> {
        let mode = match (position, kind) {
            (Position::RootValue, MapKind::Struct) => MapMode::ImplicitRoot,
            (Position::RootValue, MapKind::Dynamic) => MapMode::PendingRootMap,
            _ => {
                writer.write_char('{').map_err(fmt_error)?;
                MapMode::Explicit
            }
        };

        Ok(Self {
            writer,
            expectation: MapExpectation::Key,
            first: true,
            mode,
        })
    }

    fn finish(&mut self) -> Result {
        if self.expectation != MapExpectation::Key {
            return Err(SerializationError::custom(
                "serialize_value not called after serialize_key",
            ));
        }

        if self.mode == MapMode::Explicit {
            self.writer.write_char('}').map_err(fmt_error)?;
        }

        Ok(())
    }

    fn write_entry_separator(&mut self) -> Result {
        if !self.first {
            self.writer.write_str(", ").map_err(fmt_error)?;
        }
        Ok(())
    }
}

impl<W> ser::SerializeMap for MapSerializer<'_, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        if self.expectation != MapExpectation::Key {
            return Err(SerializationError::custom(
                "serialize_key called twice without serialize_value",
            ));
        }

        match self.mode {
            MapMode::PendingRootMap => {
                debug_assert!(self.first);
                let rendered = serialize_fragment(key, Position::MapKey)?;
                if rendered.starts_with('{') {
                    self.writer.write_char('{').map_err(fmt_error)?;
                    self.mode = MapMode::Explicit;
                } else {
                    self.mode = MapMode::ImplicitRoot;
                }
                self.writer.write_str(&rendered).map_err(fmt_error)?;
            }
            MapMode::Explicit | MapMode::ImplicitRoot => {
                self.write_entry_separator()?;
                key.serialize(Serializer {
                    writer: self.writer,
                    position: Position::MapKey,
                })?;
            }
        }

        self.expectation = MapExpectation::Value;
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        if self.expectation != MapExpectation::Value {
            return Err(SerializationError::custom(
                "serialize_value called without serialize_key",
            ));
        }

        self.writer.write_str(": ").map_err(fmt_error)?;
        value.serialize(Serializer {
            writer: self.writer,
            position: Position::MapValue,
        })?;
        self.expectation = MapExpectation::Key;
        self.first = false;
        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result {
        self.finish()
    }
}

impl<W> ser::SerializeStruct for MapSerializer<'_, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        self.write_entry_separator()?;
        write_string_or_key(self.writer, key, Position::MapKey)?;
        self.writer.write_str(": ").map_err(fmt_error)?;
        value.serialize(Serializer {
            writer: self.writer,
            position: Position::MapValue,
        })?;
        self.first = false;
        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result {
        self.finish()
    }
}

struct StructVariantSerializer<'a, W> {
    map: MapSerializer<'a, W>,
}

impl<W> ser::SerializeStructVariant for StructVariantSerializer<'_, W>
where
    W: fmt::Write,
{
    type Ok = ();
    type Error = SerializationError;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(&mut self.map, key, value)
    }

    #[inline]
    fn end(mut self) -> Result {
        self.map.finish()?;
        self.map.writer.write_char(')').map_err(fmt_error)
    }
}

fn serialize_fragment<T>(value: &T, position: Position) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut output = String::new();
    value.serialize(Serializer {
        writer: &mut output,
        position,
    })?;
    Ok(output)
}

fn write_string_or_key<W>(writer: &mut W, value: &str, position: Position) -> Result
where
    W: fmt::Write,
{
    if matches!(position, Position::MapKey) && is_valid_identifier(value) {
        write_symbol(writer, value).map_err(fmt_error)
    } else {
        write_escaped_string(writer, value).map_err(fmt_error)
    }
}

fn write_unit_variant<W>(writer: &mut W, name: &str, position: Position) -> Result
where
    W: fmt::Write,
{
    if matches!(position, Position::RootValue | Position::MapValue) && is_valid_identifier(name) {
        writer.write_str(name).map_err(fmt_error)
    } else {
        write_symbol(writer, name).map_err(fmt_error)?;
        writer.write_str("()").map_err(fmt_error)
    }
}

fn write_f32<W>(writer: &mut W, value: f32) -> Result
where
    W: fmt::Write,
{
    if value == 0.0 && value.signum() == -1.0 {
        writer.write_str("-0.0").map_err(fmt_error)
    } else if value.is_nan() {
        writer.write_str("+nan").map_err(fmt_error)
    } else if value == f32::NEG_INFINITY {
        writer.write_str("-inf").map_err(fmt_error)
    } else if value == f32::INFINITY {
        writer.write_str("+inf").map_err(fmt_error)
    } else {
        writer
            .write_str(ryu::Buffer::new().format(value))
            .map_err(fmt_error)
    }
}

fn write_f64<W>(writer: &mut W, value: f64) -> Result
where
    W: fmt::Write,
{
    if value == 0.0 && value.signum() == -1.0 {
        writer.write_str("-0.0").map_err(fmt_error)
    } else if value.is_nan() {
        writer.write_str("+nan").map_err(fmt_error)
    } else if value == f64::NEG_INFINITY {
        writer.write_str("-inf").map_err(fmt_error)
    } else if value == f64::INFINITY {
        writer.write_str("+inf").map_err(fmt_error)
    } else {
        writer
            .write_str(ryu::Buffer::new().format(value))
            .map_err(fmt_error)
    }
}

fn fmt_error(_: fmt::Error) -> SerializationError {
    SerializationError::custom("formatter write failed")
}
