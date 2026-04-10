// Update snapshot tests by running:
//
// `cargo insta test --all-features --accept`

use std::str::FromStr as _;

use eon::Value;

#[test]
fn test_parse_errors() {
    let err = eon::Value::from_str("key: $value").unwrap_err();
    let message = err.to_string().to_lowercase();
    assert!(message.contains("unexpected"));
    assert!(message.contains("byte") || message.contains("value"));

    let err = eon::Value::from_str(
        r#"
snake_case: 'ok',
kebab-case: 'forbidden'
"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("expected `:`"), "{err}");
}

#[test]
fn test_deep_recursion() {
    // Test that trying to parse a deeply nested structure fails gracefully.
    let input = "[".repeat(1000);
    let parsed = Value::from_str(input.as_str()).unwrap_err();
    let message = parsed.to_string().to_lowercase();
    assert!(message.contains("nesting depth"));
}

#[test]
fn test_bare_identifiers_are_unit_variants_by_default() {
    for name in ["nan", "inf", "nil"] {
        let parsed = Value::from_str(name).unwrap();
        let variant = parsed.as_variant().expect("expected a unit variant");
        assert_eq!(variant.name, name);
        assert!(variant.values.is_empty());
    }

    let err = Value::from_str("+NaN").unwrap_err();
    assert!(
        err.to_string().contains("NaN must be written as '+nan'"),
        "{err}"
    );
}

#[test]
fn test_repeated_key() {
    let err = Value::from_str("key: 1\nkey: 2").unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}
