#[test]
fn test_reformat() {
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

            // TODO: add more test cases here
        }
    "#;

    let formatted = con::reformat(input, &Default::default()).unwrap();
    insta::assert_snapshot!(formatted);
}
