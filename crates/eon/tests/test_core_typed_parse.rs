use std::collections::BTreeMap;

use eon::{FormatOptions, experimental};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
enum Mode {
    EnumValue,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Color {
    Rgb { r: u8, g: u8, b: u8 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Config {
    some_enum: Mode,
    label: String,
    retries: u8,
    color: Color,
    mapping: BTreeMap<Mode, Mode>,
}

fn sample_config() -> Config {
    Config {
        some_enum: Mode::EnumValue,
        label: "EnumValue".to_owned(),
        retries: 3,
        color: Color::Rgb { r: 1, g: 2, b: 3 },
        mapping: BTreeMap::from([(Mode::EnumValue, Mode::Other)]),
    }
}

#[test]
fn test_typed_core_parse_supports_compact_enum_syntax() {
    let source = r#"
        some_enum: EnumValue
        label: "EnumValue"
        retries: 3
        color: Rgb({ r: 1, g: 2, b: 3 })
        mapping: { EnumValue(): Other }
    "#;

    let parsed: Config = experimental::from_str_with_core(source).unwrap();

    assert_eq!(parsed, sample_config());
}

#[test]
fn test_typed_core_parse_roundtrips_direct_core_serializer() {
    let config = sample_config();
    let source = experimental::to_string_with_core(&config).unwrap();

    let parsed: Config = experimental::from_str_with_core(&source).unwrap();

    assert_eq!(parsed, config);
}

#[test]
fn test_typed_core_parse_accepts_current_quoted_serde_syntax() {
    let config = sample_config();
    let source = eon::to_string(&config, &FormatOptions::default()).unwrap();

    let parsed: Config = experimental::from_str_with_core(&source).unwrap();

    assert_eq!(parsed, config);
}

#[test]
fn test_typed_core_parse_root_map_matches_expected_shape() {
    let parsed: BTreeMap<String, u32> =
        experimental::from_str_with_core("alpha: 1, beta: 2").unwrap();

    assert_eq!(
        parsed,
        BTreeMap::from([("alpha".to_owned(), 1_u32), ("beta".to_owned(), 2_u32)])
    );
}

#[test]
fn test_typed_core_parse_root_implicit_list_matches_legacy_parser() {
    let parsed: Vec<u32> = experimental::from_str_with_core("1, 2 3").unwrap();

    assert_eq!(parsed, vec![1, 2, 3]);
}
