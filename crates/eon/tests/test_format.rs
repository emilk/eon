use eon::{Number, Value};

#[test]
fn test_format() {
    let value = Value::Map(
        [
            ("string".to_owned(), Value::String("a string".to_owned())),
            ("integer".to_owned(), Value::Number(Number::from(42))),
            ("float".to_owned(), Value::Number(Number::from(1.3))),
            ("boolean".to_owned(), Value::Bool(true)),
            ("nothing".to_owned(), Value::Null),
            (
                "list".to_owned(),
                Value::List(vec![
                    Value::String("item1".to_owned()),
                    Value::Number(Number::from(1337)),
                ]),
            ),
            (
                "map".to_owned(),
                Value::Map(
                    [
                        (
                            "key1".to_owned(),
                            Value::String(
                                "a string containing \"quotes\" and a newline: \n".to_owned(),
                            ),
                        ),
                        ("key2".to_owned(), Value::Number(Number::from(42.0))),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ),
        ]
        .into_iter()
        .collect(),
    );
    let formatted = value.format(&Default::default());
    insta::assert_snapshot!(formatted);
}
