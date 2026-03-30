#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use eon::{Map, Value, Variant, experimental};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
enum Mode {
    EnumValue,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct EscapedConfig {
    value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct EnumMapConfig {
    mapping: BTreeMap<Mode, u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct KeywordEnumConfig {
    mode: KeywordMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum KeywordMode {
    #[serde(rename = "true")]
    TrueKeyword,
}

#[test]
fn test_serde_map_parse_rejects_duplicate_string_keys() {
    let err = eon::from_str::<BTreeMap<String, u32>>("alpha: 1\nalpha: 2").unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_serde_map_parse_rejects_duplicate_identifier_and_quoted_aliases() {
    let err = eon::from_str::<BTreeMap<String, u32>>("alpha: 1\n\"alpha\": 2").unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_core_typed_parse_rejects_duplicate_string_keys() {
    let err = experimental::from_str_with_core::<BTreeMap<String, u32>>("alpha: 1\nalpha: 2")
        .unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_core_typed_parse_rejects_duplicate_identifier_and_quoted_aliases() {
    let err = experimental::from_str_with_core::<BTreeMap<String, u32>>("alpha: 1\n\"alpha\": 2")
        .unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_core_typed_parse_rejects_duplicate_enum_keys() {
    let err = experimental::from_str_with_core::<EnumMapConfig>(
        "mapping: { EnumValue(): 1, EnumValue(): 2 }",
    )
    .unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_serde_map_parse_rejects_duplicate_numeric_aliases() {
    let err = eon::from_str::<BTreeMap<i32, u32>>("1: 1\n1.0: 2").unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_core_typed_parse_rejects_duplicate_numeric_aliases() {
    let err = experimental::from_str_with_core::<BTreeMap<i32, u32>>("1: 1\n1.0: 2").unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_core_typed_parse_rejects_duplicate_composite_key_aliases() {
    let err = experimental::from_str_with_core::<BTreeMap<BTreeMap<String, u32>, u32>>(
        "{ { alpha: 1 }: 1, { \"alpha\": 1 }: 2 }",
    )
    .unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"), "{err}");
}

#[test]
fn test_core_typed_stringify_escapes_structure_breaking_strings() {
    let original = EscapedConfig {
        value: "\"\nadmin: true\u{1}".to_owned(),
    };

    let serialized = experimental::to_string_with_core(&original).unwrap();

    assert!(serialized.contains("\\nadmin: true"));
    assert!(serialized.contains("\\u{1}"));
    assert!(!serialized.contains("\nadmin: true"));

    let roundtripped: EscapedConfig = experimental::from_str_with_core(&serialized).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn test_core_typed_stringify_escapes_hidden_unicode_strings() {
    let original = EscapedConfig {
        value: "safe\u{200B}\u{202E}value".to_owned(),
    };

    let serialized = experimental::to_string_with_core(&original).unwrap();

    assert!(serialized.contains("\\u{200B}") || serialized.contains("\\u{200b}"));
    assert!(serialized.contains("\\u{202E}") || serialized.contains("\\u{202e}"));
    assert!(!serialized.contains('\u{200B}'));
    assert!(!serialized.contains('\u{202E}'));

    let roundtripped: EscapedConfig = experimental::from_str_with_core(&serialized).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn test_default_parse_rejects_hidden_unicode_in_literal_string_source() {
    let err = "value: \"safe\u{202E}payload\""
        .parse::<Value>()
        .unwrap_err();
    let message = err.to_string().to_lowercase();
    assert!(message.contains("invisible unicode"));
    assert!(message.contains("malicious"));
}

#[test]
fn test_default_parse_rejects_hidden_unicode_in_comments() {
    let err = eon::reformat("// hidden\u{2066}\nvalue: 1", &Default::default()).unwrap_err();
    let message = err.to_string().to_lowercase();
    assert!(message.contains("invisible unicode"));
    assert!(message.contains("malicious"));
}

#[test]
fn test_core_value_parse_rejects_hidden_unicode_in_literal_string_source() {
    let err = Value::from_str_with_core("value: \"safe\u{202E}payload\"").unwrap_err();
    let message = err.to_string().to_lowercase();
    assert!(message.contains("invisible unicode"));
    assert!(message.contains("malicious"));
}

#[test]
fn test_core_value_parse_rejects_hidden_unicode_in_comments() {
    let err = Value::from_str_with_core("// hidden\u{2066}\nvalue: 1").unwrap_err();
    let message = err.to_string().to_lowercase();
    assert!(message.contains("invisible unicode"));
    assert!(message.contains("malicious"));
}

#[test]
fn test_visible_unicode_escape_sequence_remains_allowed() {
    let parsed = "value: \"safe\\u{202E}payload\"".parse::<Value>().unwrap();
    let value = parsed.as_map().unwrap().get_str("value").unwrap();
    assert_eq!(value.as_string(), Some("safe\u{202E}payload"));
}

#[test]
fn test_core_visible_unicode_escape_sequence_remains_allowed() {
    let parsed = Value::from_str_with_core("value: \"safe\\u{202E}payload\"").unwrap();
    let value = parsed.as_map().unwrap().get_str("value").unwrap();
    assert_eq!(value.as_string(), Some("safe\u{202E}payload"));
}

#[test]
fn test_core_value_parse_does_not_attach_variant_payload_across_newline() {
    let err = Value::from_str_with_core("mode: EnumValue\n(1)").unwrap_err();
    let message = err.to_string().to_lowercase();
    assert!(message.contains("trailing") || message.contains("unexpected"));
}

#[test]
fn test_core_value_parse_does_not_attach_variant_payload_across_comment() {
    let err = Value::from_str_with_core("mode: EnumValue // hidden payload\n(1)").unwrap_err();
    let message = err.to_string().to_lowercase();
    assert!(message.contains("trailing") || message.contains("unexpected"));
}

#[test]
fn test_core_value_parse_still_allows_same_line_spacing_before_variant_payload() {
    let parsed = Value::from_str_with_core("mode: EnumValue   (1)").unwrap();
    let variant = parsed
        .as_map()
        .unwrap()
        .get_str("mode")
        .unwrap()
        .as_variant()
        .unwrap();
    assert_eq!(variant.name, "EnumValue");
    assert_eq!(variant.values, vec![Value::from(1)]);
}

#[test]
fn test_core_typed_stringify_quotes_keyword_variant_names() {
    let original = KeywordEnumConfig {
        mode: KeywordMode::TrueKeyword,
    };

    let serialized = experimental::to_string_with_core(&original).unwrap();
    assert_eq!(serialized, "mode: \"true\"()");

    let roundtripped: KeywordEnumConfig = experimental::from_str_with_core(&serialized).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn test_core_value_stringify_wraps_root_map_when_first_key_is_a_map() {
    let key = Value::Map(Map::from_iter([(Value::from("nested"), Value::from(1))]));
    let value = Value::Map(Map::from_iter([(key, Value::from("safe"))]));

    let serialized = experimental::value_to_string_with_core(&value);

    assert!(serialized.starts_with("{{"));
    let roundtripped = experimental::value_from_str_with_core(&serialized).unwrap();
    assert_eq!(roundtripped, value);
}

#[test]
fn test_core_value_parse_rejects_excessive_nesting() {
    let input = "[".repeat(1000);
    let err = Value::from_str_with_core(&input).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("nesting depth"));
}

#[test]
fn test_core_value_parse_rejects_duplicate_keys() {
    let err = Value::from_str_with_core("alpha: 1\nalpha: 2").unwrap_err();
    assert!(err.to_string().contains("Duplicate key in map"));
}

#[test]
fn test_core_value_parse_rejects_unterminated_string() {
    let err = Value::from_str_with_core("\"unterminated").unwrap_err();
    assert!(
        err.to_string()
            .to_lowercase()
            .contains("unterminated string")
    );
}

#[test]
fn test_core_stringify_preserves_variant_payload_boundaries() {
    let value = Value::Map(Map::from_iter([(
        Value::from("mode"),
        Value::Variant(Variant {
            name: "EnumValue".to_owned(),
            values: vec![Value::from("\"}\nadmin: true")],
        }),
    )]));

    let serialized = experimental::value_to_string_with_core(&value);
    assert!(serialized.contains("EnumValue("));
    assert!(serialized.contains("\\nadmin: true"));

    let roundtripped = experimental::value_from_str_with_core(&serialized).unwrap();
    assert_eq!(roundtripped, value);
}

#[test]
fn test_core_stringify_escapes_hidden_unicode_variant_names() {
    let value = Value::Map(Map::from_iter([(
        Value::from("mode"),
        Value::Variant(Variant {
            name: "Enum\u{202E}Value".to_owned(),
            values: vec![],
        }),
    )]));

    let serialized = experimental::value_to_string_with_core(&value);
    assert!(serialized.contains("\\u{202E}") || serialized.contains("\\u{202e}"));
    assert!(!serialized.contains('\u{202E}'));

    let roundtripped = experimental::value_from_str_with_core(&serialized).unwrap();
    assert_eq!(roundtripped, value);
}
