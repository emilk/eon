#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use eon::{experimental, to_value};
use serde::Serialize;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
enum Mode {
    EnumValue,
    Other,
}

#[derive(Clone, Debug, Serialize)]
enum Color {
    Rgb { r: u8, g: u8, b: u8 },
}

#[derive(Clone, Debug, Serialize)]
struct Config {
    some_enum: Mode,
    label: &'static str,
    threshold: f32,
    color: Color,
    mapping: BTreeMap<Mode, Mode>,
}

#[test]
fn test_typed_core_stringify_matches_value_core_stringify() {
    let config = Config {
        some_enum: Mode::EnumValue,
        label: "EnumValue",
        threshold: -0.0,
        color: Color::Rgb { r: 1, g: 2, b: 3 },
        mapping: BTreeMap::from([(Mode::EnumValue, Mode::Other)]),
    };

    let direct = experimental::to_string_with_core(&config).unwrap();
    let via_value = to_value(&config).unwrap().to_string_with_core();

    assert_eq!(direct, via_value);
    assert!(direct.contains("some_enum: EnumValue"));
    assert!(direct.contains("label: \"EnumValue\""));
    assert!(direct.contains("threshold: -0.0"));
    assert!(direct.contains("mapping: {EnumValue(): Other}"));
}

#[test]
fn test_typed_core_stringify_matches_value_core_for_root_map() {
    let document = BTreeMap::from([("alpha".to_owned(), 1_u32), ("beta".to_owned(), 2_u32)]);

    let direct = experimental::to_string_with_core(&document).unwrap();
    let via_value = to_value(&document).unwrap().to_string_with_core();

    assert_eq!(direct, via_value);
    assert_eq!(direct, "alpha: 1, beta: 2");
}

#[test]
fn test_typed_core_stringify_matches_value_core_for_composite_root_map_key() {
    let document = BTreeMap::from([(BTreeMap::from([("nested".to_owned(), 1_u32)]), 2_u32)]);

    let direct = experimental::to_string_with_core(&document).unwrap();
    let via_value = to_value(&document).unwrap().to_string_with_core();

    assert_eq!(direct, via_value);
    assert_eq!(direct, "{nested: 1}: 2");
}

#[test]
fn test_typed_core_stringify_keeps_empty_root_maps_explicit() {
    let document = BTreeMap::<String, u32>::new();

    let direct = experimental::to_string_with_core(&document).unwrap();
    let via_value = to_value(&document).unwrap().to_string_with_core();

    assert_eq!(direct, via_value);
    assert_eq!(direct, "{}");
}
