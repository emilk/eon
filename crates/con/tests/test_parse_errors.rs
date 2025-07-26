// Update snapshot tests by running:
//
// INSTA_FORCE_UPDATE=1 cargo t --all-features || cargo insta review

use std::str::FromStr as _;

#[test]
fn test_parse_errors() {
    insta::assert_snapshot!(con::Value::from_str("key: value").unwrap_err(), @r"
    Error:
       ╭─[ <unknown>:1:4 ]
       │
     1 │ key: value
       │    ───┬───
       │       ╰───── Unknown keyword. Expected 'null', 'true', or 'false'.
    ───╯
    ");

    insta::assert_snapshot!(con::Value::from_str("kebab-case: 'value'").unwrap_err(), @r"
    Error:
       ╭─[ <unknown>:1:6 ]
       │
     1 │ kebab-case: 'value'
       │      ──┬──
       │        ╰──── Expected end of file here
    ───╯
    ");
}
