// Update snapshot tests by running:
//
// `cargo insta test --all-features --accept`

#[test]
fn test_reformat_1() {
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

    let formatted = eon::reformat(input, &Default::default()).unwrap();
    insta::assert_snapshot!(formatted, @r#"
    // This comment is outside the outermost map.
    // This comment proceeds the first key-value pair.
    key: true // Suffix comment

    // Comment about the second key-value pair.
    // Very weird comment
    key: null
    empty_map: {}
    empty_list: []
    short_list: [1, 2, 3]
    variants: [
    	"zero_variant"
    	"one_variant"(true)
    	"three_variant"(1, 2, 3)
    	"map_variant"({
    		"key": "value"
    		"another_key": 42
    	})
    	"list_variant"([
    		"doc"
    		"grumpy"
    		"happy"
    		"sleepy"
    		"sneezy"
    		"bashful"
    		"dopey"
    	])
    ]
    "#);
}

#[test]
fn test_reformat_2() {
    let input = r#"
        suffix_commented: {
            foo: true // Suffix comment
            bar: false // Another suffix comment
        }
        prefix_commented: {
            // Commend about the first key
            foo: true
            // Comment about the second key
            bar: false
            // Closing comment
        }
    "#;

    let formatted = eon::reformat(input, &Default::default()).unwrap();
    insta::assert_snapshot!(formatted, @r"
    suffix_commented: {
    	foo: true // Suffix comment
    	bar: false // Another suffix comment
    }
    prefix_commented: {
    	// Commend about the first key
    	foo: true

    	// Comment about the second key
    	bar: false

    	// Closing comment
    }
    ");
}
