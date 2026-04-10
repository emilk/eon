use eonfmt::{FormatOptions, format_str, is_formatted};

#[test]
fn format_str_uses_default_options() {
    let formatted = format_str("key:true//comment\n").unwrap();
    assert_eq!(formatted, "key: true //comment\n");
}

#[test]
fn format_str_migrates_safe_quoted_variant_heads() {
    let formatted = format_str(r#"color: "Rgb"(255, 0, 0)"#).unwrap();
    assert_eq!(formatted, "color: Rgb(255, 0, 0)\n");
}

#[test]
fn is_formatted_reports_canonical_output() {
    let options = FormatOptions::default();

    assert!(is_formatted("key: true\n", &options).unwrap());
    assert!(!is_formatted("key:true\n", &options).unwrap());
}
