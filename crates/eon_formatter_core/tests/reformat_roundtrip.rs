use eon_formatter_core::{FormatOptions, parse_document, reformat};

fn assert_reformat_roundtrip(input: &str, expected: &str) {
    let options = FormatOptions::default();

    let once = reformat(input, &options).unwrap();
    assert_eq!(once, expected, "input:\n{input}");

    let reparsed = parse_document(&once).unwrap();
    let twice = reparsed.format(&options);
    assert_eq!(twice, once, "canonical form:\n{once}");
}

#[test]
fn single_root_value_with_trailing_comment_stays_a_value() {
    assert_reformat_roundtrip("1\n// tail\n", "1\n// tail\n");
}

#[test]
fn explicit_root_map_with_trailing_comment_stays_braceless() {
    assert_reformat_roundtrip("{ alpha: 1 } // tail\n", "alpha: 1\n\n// tail\n");
}

#[test]
fn explicit_empty_root_map_stays_explicit() {
    assert_reformat_roundtrip("{}\n", "{}");
}

#[test]
fn nested_variant_with_closing_comments_is_stable() {
    let input = r#"
        mode: EnumValue(
            {
                alpha: 1
                // map tail
            }
            // variant tail
        )
    "#;

    let once = reformat(input, &FormatOptions::default()).unwrap();
    let twice = reformat(&once, &FormatOptions::default()).unwrap();
    assert_eq!(twice, once, "canonical form:\n{once}");
}

#[test]
fn variant_map_payload_prefix_comments_are_preserved() {
    let input = r#"
        mode: EnumValue(
            // payload
            {
                alpha: 1
            }
        )
    "#;

    let expected = "mode: EnumValue(\n\t// payload\n\t{\n\t\talpha: 1\n\t}\n)\n";
    assert_reformat_roundtrip(input, expected);
}

#[test]
fn variant_list_payload_prefix_comments_are_preserved() {
    let input = r#"
        mode: EnumValue(
            // payload
            [
                1
            ]
        )
    "#;

    let expected = "mode: EnumValue(\n\t// payload\n\t[1]\n)\n";
    assert_reformat_roundtrip(input, expected);
}

#[test]
fn implicit_root_list_with_trailing_comment_is_stable() {
    let input = "1\n2\n// tail\n";
    let expected = "[\n\t1\n\t2\n\n\t// tail\n]";
    assert_reformat_roundtrip(input, expected);
}

#[test]
fn explicit_root_list_with_trailing_comment_stays_explicit() {
    let input = "[1]\n// tail\n";
    let expected = "[1]\n// tail\n";
    assert_reformat_roundtrip(input, expected);
}

#[test]
fn root_map_with_explicit_map_key_is_stable() {
    let input = "{ { alpha: 1 }: 2 }\n";
    let expected = "{\n\talpha: 1\n}: 2\n";
    assert_reformat_roundtrip(input, expected);
}

#[test]
fn root_map_with_list_key_is_stable() {
    let input = "{ [1, 2]: 3 }\n";
    let expected = "[1, 2]: 3\n";
    assert_reformat_roundtrip(input, expected);
}

#[test]
fn escaped_string_keys_are_stable() {
    let input = "{ \"line\\nfeed\": 1 }\n";
    let expected = "\"line\\nfeed\": 1\n";
    assert_reformat_roundtrip(input, expected);
}

#[test]
fn nested_strings_comments_and_composite_root_keys_are_stable() {
    let input = r#"
        { [1, 2]: {
            "line\nfeed": [
                EnumValue
                // list tail
            ]
            // map tail
        } }
    "#;

    let expected = "[1, 2]: {\n\t\"line\\nfeed\": [\n\t\tEnumValue\n\n\t\t// list tail\n\t]\n\n\t// map tail\n}\n";
    assert_reformat_roundtrip(input, expected);
}
