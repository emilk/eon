use std::fmt;

use eon_core::{is_valid_identifier, write_escaped_string, write_symbol};

use crate::{Map, Value, Variant};

/// Serialize an owned [`Value`] using the experimental compact `eon_core`
/// syntax, including bare unit variants in value position.
pub fn to_string_with_core(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value, Position::RootValue).expect("writing to String cannot fail");
    out
}

#[derive(Clone, Copy)]
enum Position {
    RootValue,
    MapKey,
    MapValue,
}

fn write_value<W>(out: &mut W, value: &Value, position: Position) -> fmt::Result
where
    W: fmt::Write,
{
    match value {
        Value::Null => out.write_str("null"),
        Value::Bool(true) => out.write_str("true"),
        Value::Bool(false) => out.write_str("false"),
        Value::Number(number) => write!(out, "{number}"),
        Value::String(string) => write_string_or_key(out, string, position),
        Value::List(list) => write_list(out, list),
        Value::Map(map) => write_map(out, map, position),
        Value::Variant(variant) => write_variant(out, variant, position),
    }
}

fn write_string_or_key<W>(out: &mut W, string: &str, position: Position) -> fmt::Result
where
    W: fmt::Write,
{
    if matches!(position, Position::MapKey) && is_valid_identifier(string) {
        write_symbol(out, string)
    } else {
        write_escaped_string(out, string)
    }
}

fn write_list<W>(out: &mut W, list: &[Value]) -> fmt::Result
where
    W: fmt::Write,
{
    out.write_char('[')?;
    for (i, value) in list.iter().enumerate() {
        if i > 0 {
            out.write_str(", ")?;
        }
        write_value(out, value, Position::MapValue)?;
    }
    out.write_char(']')
}

fn write_map<W>(out: &mut W, map: &Map, position: Position) -> fmt::Result
where
    W: fmt::Write,
{
    let implicit_root = matches!(position, Position::RootValue) && root_map_can_be_implicit(map);

    if !implicit_root {
        out.write_char('{')?;
    }

    for (i, (key, value)) in map.iter().enumerate() {
        if i > 0 {
            out.write_str(", ")?;
        }
        write_value(out, key, Position::MapKey)?;
        out.write_str(": ")?;
        write_value(out, value, Position::MapValue)?;
    }

    if !implicit_root {
        out.write_char('}')?;
    }

    Ok(())
}

fn root_map_can_be_implicit(map: &Map) -> bool {
    let Some((first_key, _)) = map.iter().next() else {
        return true;
    };

    !matches!(first_key, Value::Map(_))
}

fn write_variant<W>(out: &mut W, variant: &Variant, position: Position) -> fmt::Result
where
    W: fmt::Write,
{
    let Variant { name, values } = variant;

    let bare_unit_variant = values.is_empty()
        && matches!(position, Position::MapValue | Position::RootValue)
        && is_valid_identifier(name);

    if bare_unit_variant {
        return write_symbol(out, name);
    }

    write_symbol(out, name)?;
    out.write_char('(')?;
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            out.write_str(", ")?;
        }
        write_value(out, value, Position::MapValue)?;
    }
    out.write_char(')')
}
