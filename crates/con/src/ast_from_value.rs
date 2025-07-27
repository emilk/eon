//! The formatter uses the [`CommentedValue`] type,
//! so in order to print a [`Value`], we need a way to convert it to [`CommentedValue`].

use crate::{Value, value::Choice};

use con_syntax::{
    CommentedChoice, CommentedKeyValue, CommentedList, CommentedMap, TokenTree, TreeValue,
    escape_and_quote, key_needs_quotes,
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
                        !key_needs_quotes(key)
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
            Value::Choice(Choice { name, values }) => TreeValue::Choice(CommentedChoice {
                name_span: None,
                quoted_name: escape_and_quote(&name).into(),
                values: values.into_iter().map(Into::into).collect(),
                closing_comments: Default::default(),
            }),
        }
    }
}
