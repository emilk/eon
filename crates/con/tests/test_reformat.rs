#[test]
fn test_reformat() {
    let input = r#"
        // This comment is outside the outermost object.
        {
            // This comment proceeds the first key-value pair.
            key: "value" //Suffix comment


            // Comment about the second key-value pair.
            key:
            // Very weird comment
            42,
        }
    "#;

    let formatted = con::reformat(input, &Default::default()).unwrap();
    insta::assert_snapshot!(formatted);
}
