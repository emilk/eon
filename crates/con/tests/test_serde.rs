#![cfg(feature = "serde")]

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
struct Top {
    f: f32,
    i: i32,
    s: String,
    b: bool,
    some: Option<String>,
    none: Option<String>,
    list: Vec<String>,
    nested_object: NestedObject,
    color_a: Color,
    color_b: Color,
    tuple: (i32, String),
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
struct NestedObject {
    f: f32,
    i: i32,
    s: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
enum Color {
    Rgb { r: u8, g: u8, b: u8 },
    Hsl(u8, u8, u8),
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
        list: vec!["item1".to_owned(), "item2".to_owned()],
        nested_object: NestedObject {
            f: 3.2,
            i: 7,
            s: "Nested".to_owned(),
        },
        color_a: Color::Rgb { r: 255, g: 0, b: 0 },
        color_b: Color::Hsl(0, 100, 200),
        tuple: (100, "Tuple".to_owned()),
    };

    let string = con::to_string(&top, &con::FormatOptions::default()).unwrap();
    insta::assert_snapshot!(string);

    // let roundtripped: Top = con::from_str(&string).unwrap(); // TODO
    // assert_eq!(top, roundtripped);
}
