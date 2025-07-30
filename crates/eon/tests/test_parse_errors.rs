// Update snapshot tests by running:
//
// `cargo insta test --all-features --accept`

use std::str::FromStr as _;

use eon::Value;

#[test]
fn test_parse_errors() {
    insta::assert_snapshot!(eon::Value::from_str("key: value").unwrap_err(), @r#"
    Error:
       ╭─[ <unknown>:1:6 ]
       │
     1 │ key: value
       │      ──┬──
       │        ╰──── Unknown keyword "value". Expected 'null', 'true', or 'false'.
    ───╯
    "#);

    // TODO(emilk): improve this error message
    insta::assert_snapshot!(eon::Value::from_str(
        r"
snake_case: 'ok',
kebab-case: 'forbidden'
").unwrap_err(), @r"
    Error:
       ╭─[ <unknown>:3:6 ]
       │
     3 │ kebab-case: 'forbidden'
       │      ──┬──
       │        ╰──── Expected colon ':' but found number
    ───╯
    ");
}

#[test]
fn test_deep_recursion() {
    // Test that trying to parse a deeply nested structure fails gracefully.
    let input = "[".repeat(1000);
    let parsed = Value::from_str(input.as_str()).unwrap_err();
    insta::assert_snapshot!(parsed, @r"
    Error:
       ╭─[ <unknown>:1:64 ]
       │
     1 │ [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
       │                                                                ┬
       │                                                                ╰── Maximum recursion depth exceeded while parsing document
    ───╯
    ");
}

#[test]
fn test_almost_correct() {
    insta::assert_snapshot!(Value::from_str("nan").unwrap_err(), @r#"
    Error:
       ╭─[ <unknown>:1:1 ]
       │
     1 │ nan
       │ ─┬─
       │  ╰─── Unknown keyword "nan". Did you mean: +nan?
    ───╯
    "#);

    insta::assert_snapshot!(Value::from_str("inf").unwrap_err(), @r#"
    Error:
       ╭─[ <unknown>:1:1 ]
       │
     1 │ inf
       │ ─┬─
       │  ╰─── Unknown keyword "inf". Did you mean: +inf or -inf?
    ───╯
    "#);

    insta::assert_snapshot!(Value::from_str("+NaN").unwrap_err(), @r#"
    Error:
       ╭─[ <unknown>:1:1 ]
       │
     1 │ +NaN
       │ ──┬─
       │   ╰─── Failed to parse number: NaN must be written as '+nan'. The string: "+NaN"
    ───╯
    "#);

    insta::assert_snapshot!(Value::from_str("nil").unwrap_err(), @r#"
    Error:
       ╭─[ <unknown>:1:1 ]
       │
     1 │ nil
       │ ─┬─
       │  ╰─── Unknown keyword "nil". Did you mean: null?
    ───╯
    "#);
}
