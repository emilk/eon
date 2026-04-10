use eon_formatter_core::{FormatOptions, parse_document, reformat};

#[test]
fn explicit_root_maps_canonicalize_to_braceless_form_by_default() {
    let formatted = reformat("{ alpha: 1, beta: 2 }", &FormatOptions::default()).unwrap();
    assert_eq!(formatted, "alpha: 1\nbeta: 2\n");
}

#[test]
fn always_include_outer_braces_preserves_root_map_container() {
    let document = parse_document("{ alpha: 1, beta: 2 }").unwrap();
    let options = FormatOptions {
        always_include_outer_braces: true,
        ..FormatOptions::default()
    };

    let formatted = document.format(&options);
    assert_eq!(formatted, "{\n\talpha: 1\n\tbeta: 2\n}");
}

#[test]
fn composite_root_keys_remain_braceless_by_default() {
    let formatted = reformat("{ [1, 2]: 3 }", &FormatOptions::default()).unwrap();
    assert_eq!(formatted, "[1, 2]: 3\n");
}

#[test]
fn always_include_outer_braces_can_force_composite_root_keys() {
    let document = parse_document("{ [1, 2]: 3 }").unwrap();
    let options = FormatOptions {
        always_include_outer_braces: true,
        ..FormatOptions::default()
    };

    let formatted = document.format(&options);
    assert_eq!(formatted, "{\n\t[1, 2]: 3\n}");
}

#[test]
fn whitespace_between_key_and_colon_is_canonicalized() {
    let formatted = reformat("alpha\n: 1", &FormatOptions::default()).unwrap();
    assert_eq!(formatted, "alpha: 1\n");
}

#[test]
fn comments_between_key_and_colon_are_rejected() {
    let err = reformat("alpha // nope\n: 1", &FormatOptions::default()).unwrap_err();
    assert!(err.to_string().contains("unexpected ':'"));
}

#[test]
fn comments_between_colon_and_value_become_entry_prefix_comments() {
    let formatted = reformat("alpha:\n// value comment\n1", &FormatOptions::default()).unwrap();
    assert_eq!(formatted, "// value comment\nalpha: 1\n");
}

#[test]
fn bare_identifier_unit_variants_are_a_supported_extension() {
    let formatted = reformat("mode: EnumValue", &FormatOptions::default()).unwrap();
    assert_eq!(formatted, "mode: EnumValue\n");
}

#[test]
fn bare_identifier_payload_variants_are_a_supported_extension() {
    let formatted = reformat("mode: EnumValue({ answer: 42 })", &FormatOptions::default()).unwrap();
    assert_eq!(formatted, "mode: EnumValue({\n\tanswer: 42\n})\n");
}

#[test]
fn quoted_variant_heads_remain_supported() {
    let formatted = reformat(
        "mode: \"Quoted\"({ answer: 42 })",
        &FormatOptions::default(),
    )
    .unwrap();
    assert_eq!(formatted, "mode: Quoted({\n\tanswer: 42\n})\n");
}

#[test]
fn variant_payload_parentheses_must_remain_inline() {
    let err = reformat("\"Rgb\" // nope\n({ r: 1 })", &FormatOptions::default()).unwrap_err();
    assert!(err.to_string().contains("unexpected '('"));
}
