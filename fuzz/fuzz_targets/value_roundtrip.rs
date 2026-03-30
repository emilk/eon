#![no_main]

use arbitrary::Unstructured;
use eon::{Map, Value, Variant};
use libfuzzer_sys::fuzz_target;

const MAX_DEPTH: u8 = 4;
const MAX_LEN: usize = 4;

fuzz_target!(|data: &[u8]| {
    let Ok(value) = arbitrary_value(&mut Unstructured::new(data), MAX_DEPTH) else {
        return;
    };

    if supports_exact_core_roundtrip(&value, false) {
        let core_text = value.to_string_with_core();
        let reparsed_core = Value::from_str_with_core(&core_text)
            .expect("core serializer output should parse with the core parser");
        assert_eq!(reparsed_core, value);
    }

    if supports_exact_legacy_roundtrip(&value, false) {
        let legacy_text = value.format(&Default::default());
        let reparsed_legacy: Value = legacy_text
            .parse()
            .expect("legacy formatter output should parse with the legacy parser");
        assert_eq!(reparsed_legacy, value);
    }
});

fn arbitrary_value(u: &mut Unstructured<'_>, depth: u8) -> arbitrary::Result<Value> {
    let branch = if depth == 0 {
        u.int_in_range(0..=4)?
    } else {
        u.int_in_range(0..=6)?
    };

    Ok(match branch {
        0 => Value::Null,
        1 => Value::Bool(u.arbitrary()?),
        2 => arbitrary_number(u),
        3 => Value::from(arbitrary_string(u)?),
        4 => Value::Variant(Variant {
            name: arbitrary_variant_name(u),
            values: arbitrary_children(u, depth.saturating_sub(1))?,
        }),
        5 => Value::List(arbitrary_children(u, depth.saturating_sub(1))?),
        _ => Value::Map(arbitrary_map(u, depth.saturating_sub(1))?),
    })
}

fn arbitrary_children(u: &mut Unstructured<'_>, depth: u8) -> arbitrary::Result<Vec<Value>> {
    let len = u.int_in_range(0..=MAX_LEN)?;
    (0..len).map(|_| arbitrary_value(u, depth)).collect()
}

fn arbitrary_map(u: &mut Unstructured<'_>, depth: u8) -> arbitrary::Result<Map> {
    let len = u.int_in_range(0..=MAX_LEN)?;
    let mut map = Map::with_capacity(len);
    for _ in 0..len {
        let key = arbitrary_value(u, depth)?;
        let value = arbitrary_value(u, depth)?;
        map.insert(key, value);
    }
    Ok(map)
}

fn arbitrary_number(u: &mut Unstructured<'_>) -> Value {
    match u.int_in_range(0..=8).unwrap_or(0) {
        0 => Value::from(u.arbitrary::<i64>().unwrap_or_default()),
        1 => Value::from(u.arbitrary::<u64>().unwrap_or_default()),
        2 => Value::from(u.arbitrary::<i16>().unwrap_or_default()),
        3 => Value::from(u.arbitrary::<u16>().unwrap_or_default()),
        4 => Value::from(f32::NAN),
        5 => Value::from(f32::INFINITY),
        6 => Value::from(f32::NEG_INFINITY),
        7 => Value::from(-0.0_f64),
        _ => Value::from(u.arbitrary::<f64>().unwrap_or_default()),
    }
}

fn arbitrary_string(u: &mut Unstructured<'_>) -> arbitrary::Result<String> {
    let len = u.int_in_range(0..=MAX_LEN)?;
    let mut out = String::new();
    for _ in 0..len {
        out.push(u.arbitrary::<char>()?);
    }
    Ok(out)
}

fn arbitrary_variant_name(u: &mut Unstructured<'_>) -> String {
    let name = arbitrary_string(u).unwrap_or_default();
    if name.is_empty() { "_".to_owned() } else { name }
}

fn supports_exact_legacy_roundtrip(value: &Value, in_map_key: bool) -> bool {
    match value {
        // The legacy formatter emits bare keywords for bool/null values, but
        // those cannot roundtrip exactly in map-key position. It also renders
        // empty variants as quoted strings, so unit variants are excluded.
        Value::Null | Value::Bool(_) => !in_map_key,
        Value::Number(_) | Value::String(_) => true,
        Value::List(values) => values
            .iter()
            .all(|value| supports_exact_legacy_roundtrip(value, false)),
        Value::Map(map) => map.iter().all(|(key, value)| {
            supports_exact_legacy_roundtrip(key, true)
                && supports_exact_legacy_roundtrip(value, false)
        }),
        Value::Variant(variant) => {
            !variant.values.is_empty()
                && variant
                    .values
                    .iter()
                    .all(|value| supports_exact_legacy_roundtrip(value, false))
        }
    }
}

fn supports_exact_core_roundtrip(value: &Value, in_map_key: bool) -> bool {
    match value {
        // The compact core parser canonicalizes keyword-looking map keys to
        // strings ("null"/"true"/"false"), so exact roundtrip excludes
        // bool/null values in key position.
        Value::Null | Value::Bool(_) => !in_map_key,
        Value::Number(_) | Value::String(_) => true,
        Value::List(values) => values
            .iter()
            .all(|value| supports_exact_core_roundtrip(value, false)),
        Value::Map(map) => map.iter().all(|(key, value)| {
            supports_exact_core_roundtrip(key, true) && supports_exact_core_roundtrip(value, false)
        }),
        Value::Variant(variant) => variant
            .values
            .iter()
            .all(|value| supports_exact_core_roundtrip(value, false)),
    }
}
