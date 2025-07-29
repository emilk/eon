// Update snapshot tests by running:
//
// `cargo insta test --all-features --accept`

use std::str::FromStr as _;

#[test]
fn test_parse_errors() {
    insta::assert_snapshot!(con::Value::from_str("key: value").unwrap_err(), @r#"
    Error:
       ╭─[ <unknown>:1:6 ]
       │
     1 │ key: value
       │      ──┬──
       │        ╰──── Unknown keyword "value". Expected 'null', 'true', or 'false'.
    ───╯
    "#);

    // TODO(emilk): improve this error message
    insta::assert_snapshot!(con::Value::from_str(
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
