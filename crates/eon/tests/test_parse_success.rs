// Update snapshot tests by running:
//
// `cargo insta test --all-features --accept`

use std::str::FromStr as _;

use eon::Value;

#[test]
fn test_deep_object() {
    let input = r#"
        {"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{"":{}}}}}}}}}}}}}}}}}}}}}}}
    "#;
    let parsed = Value::from_str(input);
    assert!(parsed.is_ok());
}

#[test]
fn test_unicode() {
    let input = r#"
        "key": "value"
        "emoji": "😀"
        "cyrillic": "Привет мир!"
        "chinese": "你好，世界！"
        "arabic": "مرحبا بالعالم"
        "horrid escaping": "\\\\\\\"\\\\\\"
        "Rust-style unicode": "\u{1f6ad}"
    "#;
    let parsed = Value::from_str(input);
    let parsed = parsed.unwrap_or_else(|err| panic!("Failed to parse unicode test input: {err}"));
    insta::assert_snapshot!(parsed.to_string(), @r#"
    "key": "value"
    "emoji": "😀"
    "cyrillic": "Привет мир!"
    "chinese": "你好，世界！"
    "arabic": "مرحبا بالعالم"
    "horrid escaping": '\\\\\\"\\\\\\'
    "Rust-style unicode": "🚭"
    "#);
}
