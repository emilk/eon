#![cfg(feature = "serde")]

use std::collections::BTreeMap;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
struct Top {
    f: f32,
    i: i32,
    s: String,
    b: bool,
    some: Option<String>,
    none: Option<String>,
    floats: Vec<f32>,
    nested_object: NestedObject,
    colors: Vec<Color>,
    tuple: (i32, String),
    map: BTreeMap<i32, f32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
struct NestedObject {
    f: f32,
    i: i32,
    s: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
enum Color {
    Black,
    Gray(u8),
    Hsl(u8, u8, u8),
    Rgb { r: u8, g: u8, b: u8 },
}

#[test]
fn test_serde() {
    let top = Top {
        f: 1.23,
        i: 42,
        s: "Hello, world!".to_owned(),
        b: true,
        some: Some("Some".to_owned()),
        none: None,
        floats: vec![1.1, f32::NEG_INFINITY, f32::INFINITY],
        nested_object: NestedObject {
            f: 3.2,
            i: 7,
            s: "Nested".to_owned(),
        },
        colors: vec![
            Color::Black,
            Color::Gray(128),
            Color::Hsl(0, 100, 200),
            Color::Rgb { r: 255, g: 0, b: 0 },
        ],
        tuple: (100, "Tuple".to_owned()),
        map: BTreeMap::from([(1, 1.1), (2, f32::NEG_INFINITY), (3, f32::INFINITY)]),
    };

    let string = eon::to_string(&top, &eon::FormatOptions::default()).unwrap();
    insta::assert_snapshot!(string);

    let roundtripped: Top = eon::from_str(&string).unwrap();
    assert_eq!(top, roundtripped);
}

#[test]
fn test_string_to_bool() {
    let bool_map: BTreeMap<String, bool> =
        BTreeMap::from([("true".to_owned(), true), ("false".to_owned(), false)]);
    let result = eon::to_string(&bool_map, &eon::FormatOptions::default());
    insta::assert_snapshot!(result.unwrap(), @r#"
    "false": false
    "true": true
    "#);
}

#[test]
fn test_bool_to_string() {
    let bool_map: BTreeMap<bool, String> =
        BTreeMap::from([(true, "true".to_owned()), (false, "false".to_owned())]);
    let result = eon::to_string(&bool_map, &eon::FormatOptions::default());
    insta::assert_snapshot!(result.unwrap(), @r#"
    false: "false"
    true: "true"
    "#);
}

#[test]
fn test_serde_roundtrips_nul_escape_in_strings() {
    #[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
    struct WithNul {
        value: String,
    }

    let original = WithNul {
        value: "prefix\0suffix".to_owned(),
    };

    let serialized = eon::to_string(&original, &eon::FormatOptions::default()).unwrap();
    assert!(serialized.contains("\\0"));

    let roundtripped: WithNul = eon::from_str(&serialized).unwrap();
    assert_eq!(roundtripped, original);
}

#[test]
fn test_serde_escapes_hidden_unicode_in_strings() {
    #[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
    struct HiddenUnicode {
        value: String,
    }

    let original = HiddenUnicode {
        value: "prefix\u{11101}suffix".to_owned(),
    };

    let serialized = eon::to_string(&original, &eon::FormatOptions::default()).unwrap();
    assert!(
        serialized.contains("\\u{11101}")
            || serialized.contains("\\u{11101}".to_lowercase().as_str())
    );
    assert!(!serialized.contains('\u{11101}'));

    let roundtripped: HiddenUnicode = eon::from_str(&serialized).unwrap();
    assert_eq!(roundtripped, original);
}
