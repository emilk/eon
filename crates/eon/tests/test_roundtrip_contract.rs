use eon::{Map, Value, Variant, experimental};

#[test]
fn test_core_value_roundtrip_keeps_empty_root_maps_explicit() {
    let original = Value::Map(Map::new());

    let serialized = experimental::value_to_string_with_core(&original);
    assert_eq!(serialized, "{}");

    let parsed = experimental::value_from_str_with_core(&serialized).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn test_core_value_roundtrip_keeps_composite_root_map_keys_braceless() {
    let original = Value::Map(Map::from_iter([(
        Value::Map(Map::from_iter([(Value::from("nested"), Value::from(1))])),
        Value::from(2),
    )]));

    let serialized = experimental::value_to_string_with_core(&original);
    assert_eq!(serialized, "{nested: 1}: 2");

    let parsed = experimental::value_from_str_with_core(&serialized).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn test_core_value_roundtrip_excludes_keyword_map_keys() {
    let original = Value::Map(Map::from_iter([
        (Value::Null, Value::from(0)),
        (Value::Bool(true), Value::from(1)),
        (Value::Bool(false), Value::from(2)),
    ]));

    let serialized = experimental::value_to_string_with_core(&original);
    assert_eq!(serialized, "null: 0, true: 1, false: 2");

    let parsed = experimental::value_from_str_with_core(&serialized).unwrap();
    let expected = Value::Map(Map::from_iter([
        (Value::from("null"), Value::from(0)),
        (Value::from("true"), Value::from(1)),
        (Value::from("false"), Value::from(2)),
    ]));

    assert_eq!(parsed, expected);
    assert_ne!(parsed, original);
}

#[test]
fn test_legacy_value_roundtrip_excludes_keyword_map_keys() {
    let original = Value::Map(Map::from_iter([
        (Value::Null, Value::from(0)),
        (Value::Bool(true), Value::from(1)),
        (Value::Bool(false), Value::from(2)),
    ]));

    let serialized = original.format(&Default::default());
    assert_eq!(serialized, "null: 0\ntrue: 1\nfalse: 2\n");

    let parsed: Value = serialized.parse().unwrap();
    let expected = Value::Map(Map::from_iter([
        (Value::from("null"), Value::from(0)),
        (Value::from("true"), Value::from(1)),
        (Value::from("false"), Value::from(2)),
    ]));

    assert_eq!(parsed, expected);
    assert_ne!(parsed, original);
}

#[test]
fn test_legacy_value_roundtrip_excludes_unit_variants() {
    let original = Value::Variant(Variant {
        name: "EnumValue".to_owned(),
        values: vec![],
    });

    let serialized = original.format(&Default::default());
    assert_eq!(serialized, "\"EnumValue\"");

    let parsed: Value = serialized.parse().unwrap();
    assert_eq!(parsed, Value::from("EnumValue"));
    assert_ne!(parsed, original);
}
