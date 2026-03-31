use eonfmt::{FormatOptions, format_str, is_formatted};

#[test]
fn format_str_uses_default_options() {
    let formatted = format_str("key:true//comment\n").unwrap();
    assert_eq!(formatted, "key: true //comment\n");
}

#[test]
fn is_formatted_reports_canonical_output() {
    let options = FormatOptions::default();

    assert!(is_formatted("key: true\n", &options).unwrap());
    assert!(!is_formatted("key:true\n", &options).unwrap());
}
