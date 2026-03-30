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
