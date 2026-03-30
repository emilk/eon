use eon_formatter_core::{FormatOptions, reformat};

fn assert_reformat_matches_legacy(input: &str) {
    let ours = reformat(input, &FormatOptions::default()).unwrap();
    let legacy = eon_syntax::reformat(input, &eon_syntax::FormatOptions::default()).unwrap();
    assert_eq!(ours, legacy, "input:\n{input}");
}

#[test]
fn matches_legacy_on_complex_root_map() {
    let input = r#"
        // This comment is outside the outermost map.
        {
            // This comment proceeds the first key-value pair.
            key: true// Suffix comment


            // Comment about the second key-value pair.
            key:
            // Very weird comment
            null

            empty_map: {}
            empty_list: []
            short_list: [1, 2, 3]

            variants: [
                "zero_variant"()
                "one_variant"(true)
                "three_variant"(1, 2, 3)
                "map_variant"({
                    "key": "value",
                    "another_key": 42,
                })
                "list_variant"([
                    "doc",
                    "grumpy",
                    "happy",
                    "sleepy",
                    "sneezy",
                    "bashful",
                    "dopey",
                ])
            ]
        }
    "#;

    assert_reformat_matches_legacy(input);
}

#[test]
fn matches_legacy_on_suffix_and_prefix_comments() {
    let input = r#"
        suffix_commented: {
            foo: true // Suffix comment
            bar: false // Another suffix comment
        }
        prefix_commented: {
            // Comment about the first key
            foo: true
            // Comment about the second key
            bar: false
            // Closing comment
        }
    "#;

    assert_reformat_matches_legacy(input);
}

#[test]
fn matches_legacy_on_root_scalar_and_root_list() {
    assert_reformat_matches_legacy("\"Mode\"({ flag: true })");
    assert_reformat_matches_legacy("[1, 2, 3]");
}
