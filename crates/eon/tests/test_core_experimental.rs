use eon::{Value, experimental::value_from_str_with_core};

#[test]
fn test_core_parser_distinguishes_unit_variant_from_string() {
    let parsed = value_from_str_with_core("some_enum: EnumValue").unwrap();
    let map = parsed.as_map().unwrap();
    let value = map.get_str("some_enum").unwrap();
    let variant = value.as_variant().unwrap();

    assert_eq!(variant.name, "EnumValue");
    assert!(variant.values.is_empty());
}

#[test]
fn test_core_parser_keeps_quoted_value_as_string() {
    let parsed = value_from_str_with_core("some_enum: \"EnumValue\"").unwrap();
    let map = parsed.as_map().unwrap();
    let value = map.get_str("some_enum").unwrap();

    assert_eq!(value.as_string(), Some("EnumValue"));
    assert!(value.as_variant().is_none());
}

#[test]
fn test_core_parser_supports_payload_variants() {
    let parsed = value_from_str_with_core("some_enum: EnumValue(42, { nested: true })").unwrap();
    let map = parsed.as_map().unwrap();
    let value = map.get_str("some_enum").unwrap();
    let variant = value.as_variant().unwrap();

    assert_eq!(variant.name, "EnumValue");
    assert_eq!(variant.values.len(), 2);
    assert_eq!(variant.values[0], Value::from(42));
    assert!(variant.values[1].as_map().is_some());
}

#[test]
fn test_core_parser_explicit_unit_variant_matches_bare_identifier() {
    let bare = value_from_str_with_core("some_enum: EnumValue").unwrap();
    let explicit = value_from_str_with_core("some_enum: EnumValue()").unwrap();

    assert_eq!(bare, explicit);
}
