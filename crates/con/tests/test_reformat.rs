#[test]
fn test_reformat() {
    let input = r#"
        // This comment is outside the outermost object.
        {
            // This comment proceeds the first key-value pair.
            key: true// Suffix comment


            // Comment about the second key-value pair.
            key:
            // Very weird comment
            null
        }
    "#;

    let formatted = con::reformat(input, &Default::default()).unwrap();
    insta::assert_snapshot!(formatted);
}
