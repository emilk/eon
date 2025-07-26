//! The formatter uses the [`CommentedValue`] type,
//! so in order to print a [`Value`], we need a way to convert it to [`CommentedValue`].

use crate::Value;

use con_syntax::{
    CommentedChoice, CommentedKeyValue, CommentedList, CommentedMap, TokenTree, TreeValue,
    needs_quotes,
};

impl From<Value> for TokenTree<'static> {
    fn from(value: Value) -> Self {
        Self::from(TreeValue::from(value))
    }
}

impl From<Value> for TreeValue<'static> {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => TreeValue::Identifier("null".into()),
            Value::Bool(true) => TreeValue::Identifier("true".into()),
            Value::Bool(false) => TreeValue::Identifier("false".into()),
            Value::Number(number) => TreeValue::Number(number.to_string().into()),
            Value::String(string) => TreeValue::QuotedString(escape_and_quote(&string).into()),
            Value::List(list) => TreeValue::List(CommentedList {
                values: list.into_iter().map(Into::into).collect(),
                closing_comments: vec![],
            }),
            Value::Map(map) => {
                let all_keys_are_identifiers = map.iter().all(|(key, _)| {
                    if let Value::String(key) = key {
                        !needs_quotes(key)
                    } else {
                        false
                    }
                });

                TreeValue::Map(CommentedMap {
                    key_values: map
                        .into_iter()
                        .map(|(key, value)| {
                            let key = if all_keys_are_identifiers {
                                if let Value::String(key) = key {
                                    TokenTree::from(TreeValue::Identifier(key.into()))
                                } else {
                                    unreachable!();
                                }
                            } else {
                                TokenTree::from(key)
                            };

                            CommentedKeyValue {
                                key,
                                value: TokenTree::from(value),
                            }
                        })
                        .collect(),
                    closing_comments: Default::default(),
                })
            }
            Value::Choice(choice) => TreeValue::Choice(CommentedChoice {
                name_span: Default::default(), // TODO
                name: choice.name.into(),
                values: choice.values.into_iter().map(Into::into).collect(),
                closing_comments: Default::default(),
            }),
        }
    }
}

fn escape_and_quote(raw: &str) -> String {
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
    assert_eq!(escape_and_quote("normal"), r#""normal""#);
    assert_eq!(
        escape_and_quote(r#"a string with 'single-quotes'"#),
        r#""a string with 'single-quotes'""#
    );
    assert_eq!(
        escape_and_quote(r#"a string with "double-quotes""#),
        r#"'a string with "double-quotes"'"#
    );
    assert_eq!(
        escape_and_quote(r#"a string with 'single-quotes' and "double-quotes""#),
        r#""a string with 'single-quotes' and \"double-quotes\"""#
    );
    assert_eq!(
        escape_and_quote("a string with newline \n"),
        r#""a string with newline \n""#
    );
}
