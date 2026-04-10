use eon::{Map, Value, Variant, experimental::value_to_string_with_core};

#[test]
fn test_core_stringify_uses_bare_unit_variant_values() {
    let value = Value::Map(Map::from_iter([(
        Value::from("some_enum"),
        Value::Variant(Variant {
            name: "EnumValue".to_owned(),
            values: vec![],
        }),
    )]));

    assert_eq!(value_to_string_with_core(&value), "some_enum: EnumValue");
}

#[test]
fn test_core_stringify_keeps_unit_variant_keys_explicit() {
    let value = Value::Map(Map::from_iter([(
        Value::Variant(Variant {
            name: "EnumKey".to_owned(),
            values: vec![],
        }),
        Value::Variant(Variant {
            name: "EnumValue".to_owned(),
            values: vec![],
        }),
    )]));

    assert_eq!(value_to_string_with_core(&value), "EnumKey(): EnumValue");
}

#[test]
fn test_core_stringify_roundtrips_through_core_parser() {
    let original = Value::Map(Map::from_iter([
        (
            Value::from("some_enum"),
            Value::Variant(Variant {
                name: "EnumValue".to_owned(),
                values: vec![Value::from(42), Value::from("hello")],
            }),
        ),
        (
            Value::Variant(Variant {
                name: "EnumKey".to_owned(),
                values: vec![],
            }),
            Value::Variant(Variant {
                name: "Other".to_owned(),
                values: vec![],
            }),
        ),
    ]));

    let string = value_to_string_with_core(&original);
    let parsed = Value::from_str_with_core(&string).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn test_core_stringify_quotes_non_identifier_unit_variants() {
    let value = Value::Variant(Variant {
        name: "kebab-case".to_owned(),
        values: vec![],
    });

    assert_eq!(value_to_string_with_core(&value), "\"kebab-case\"()");
}

#[test]
fn test_core_stringify_keeps_empty_root_maps_explicit() {
    let value = Value::Map(Map::new());

    assert_eq!(value_to_string_with_core(&value), "{}");
}
