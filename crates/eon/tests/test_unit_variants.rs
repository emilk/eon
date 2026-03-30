use std::str::FromStr as _;

use eon::Value;

#[test]
fn test_explicit_unit_variant_parses_as_variant() {
    let parsed = Value::from_str("\"EnumValue\"()").unwrap();
    let variant = parsed.as_variant().unwrap();

    assert_eq!(variant.name, "EnumValue");
    assert!(variant.values.is_empty());
}

#[test]
fn test_unit_variant_formats_like_current_canonical_syntax() {
    let parsed = Value::from_str("\"EnumValue\"()").unwrap();

    assert_eq!(parsed.to_string(), "\"EnumValue\"");
}
