// Update snapshot tests by running:
//
// INSTA_FORCE_UPDATE=1 cargo t --all-features || cargo insta review

use std::str::FromStr as _;

#[test]
fn test_parse_errors() {
    // TODO: improve the span of this error
    insta::assert_snapshot!(con::Value::from_str("key: value").unwrap_err(), @r"
    Error:
       ╭─[ <unknown>:1:4 ]
       │
     1 │ key: value
       │    ───┬───
       │       ╰───── Unknown keyword. Expected 'null', 'true', or 'false'.
    ───╯
    ");

    // TODO: improve this error message
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
