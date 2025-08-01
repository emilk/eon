use std::str::FromStr as _;

#[test]
fn test_example_eon() {
    let example = include_str!("../../../example.eon");
    let formatted = eon::reformat(example, &eon::FormatOptions::default()).unwrap();
    assert_eq!(
        formatted, example,
        "example.eon would be reformatted by eon::format"
    );
    let parsed = eon::Value::from_str(example).unwrap();
    let multiline_basic_strings = parsed
        .as_map()
        .expect("example.eon should be a map")
        .get_str("multiline_basic_strings")
        .expect("multiline_basic_strings should exist");
    let multiline_basic_strings = multiline_basic_strings.as_list().expect("should be a list");
    assert_eq!(multiline_basic_strings.len(), 2,);
    assert_eq!(multiline_basic_strings[0], multiline_basic_strings[1]);
}
