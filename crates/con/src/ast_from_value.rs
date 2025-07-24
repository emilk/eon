//! The formatter uses the [`CommentedValue`] type,
//! so in order to print a [`Value`], we need a way to convert it to [`CommentedValue`].

use crate::{
    Value,
    ast::{AstValue, CommentedKeyValue, CommentedList, CommentedObject, CommentedValue},
};

impl From<Value> for CommentedValue<'static> {
    fn from(value: Value) -> Self {
        CommentedValue {
            span: Default::default(),
            prefix_comments: vec![],
            value: AstValue::from(value),
            suffix_comment: None,
        }
    }
}

impl From<Value> for AstValue<'static> {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => AstValue::Identifier("null".into()),
            Value::Bool(true) => AstValue::Identifier("true".into()),
            Value::Bool(false) => AstValue::Identifier("false".into()),
            Value::Number(number) => AstValue::Number(number.to_string().into()),
            Value::String(string) => AstValue::QuotedString(escape(&string).into()),
            Value::List(list) => AstValue::List(CommentedList {
                values: list.into_iter().map(Into::into).collect(),
                closing_comments: vec![],
            }),
            Value::Object(object) => AstValue::Object(CommentedObject {
                key_values: object
                    .into_iter()
                    .map(|(key, value)| CommentedKeyValue {
                        key: AstValue::Identifier(key.into()),
                        value: AstValue::from(value),
                        prefix_comments: Default::default(),
                        suffix_comment: Default::default(),
                    })
                    .collect(),
                closing_comments: Default::default(),
            }),
        }
    }
}

fn escape(raw: &str) -> String {
    #![expect(clippy::unwrap_used)] // Can't fail - the Debug format always produces a string with quotes.

    let escaped = format!("{raw:?}");

    if raw.contains('"') && !raw.contains('\'') {
        // Use single-quotes instead of double-quotes.
        let unquoted = escaped
            .strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap(); // Remove quotes
        let unquoted = unquoted.replace("\\\"", "\""); // No need to escape double-quotes
        format!("'{unquoted}'") // Use single-quotes
    } else {
        escaped
    }
}

#[test]
fn test_escape() {
    assert_eq!(escape("normal"), r#""normal""#);
    assert_eq!(
        escape(r#"a string with 'single-quotes'"#),
        r#""a string with 'single-quotes'""#
    );
    assert_eq!(
        escape(r#"a string with "double-quotes""#),
        r#"'a string with "double-quotes"'"#
    );
    assert_eq!(
        escape(r#"a string with 'single-quotes' and "double-quotes""#),
        r#""a string with 'single-quotes' and \"double-quotes\"""#
    );
    assert_eq!(
        escape("a string with newline \n"),
        r#""a string with newline \n""#
    );
}
