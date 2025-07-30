//! The [`eon_syntax`] crate uses the [`TokenTree`] type as inpuit to the formatter,
//! so in order to format a [`Value`] as we need a way to convert it to [`TokenValue`].
//! That's the purpose of this module.

use crate::{Value, value::Variant};

use eon_syntax::{
    TokenKeyValue, TokenList, TokenMap, TokenTree, TokenValue, TokenVariant, escape_and_quote,
    key_needs_quotes,
};

impl From<Value> for TokenTree<'static> {
    fn from(value: Value) -> Self {
        Self::from(TokenValue::from(value))
    }
}

impl From<Value> for TokenValue<'static> {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => TokenValue::Identifier("null".into()),
            Value::Bool(true) => TokenValue::Identifier("true".into()),
            Value::Bool(false) => TokenValue::Identifier("false".into()),
            Value::Number(number) => TokenValue::Number(number.to_string().into()),
            Value::String(string) => TokenValue::QuotedString(escape_and_quote(&string).into()),
            Value::List(list) => TokenValue::List(TokenList {
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

                TokenValue::Map(TokenMap {
                    key_values: map
                        .into_iter()
                        .map(|(key, value)| {
                            let key = if all_keys_are_identifiers {
                                if let Value::String(key) = key {
                                    TokenTree::from(TokenValue::Identifier(key.into()))
                                } else {
                                    unreachable!();
                                }
                            } else {
                                TokenTree::from(key)
                            };

                            TokenKeyValue {
                                key,
                                value: TokenTree::from(value),
                            }
                        })
                        .collect(),
                    closing_comments: Default::default(),
                })
            }
            Value::Variant(Variant { name, values }) => TokenValue::Variant(TokenVariant {
                name_span: None,
                quoted_name: escape_and_quote(&name).into(),
                values: values.into_iter().map(Into::into).collect(),
                closing_comments: Default::default(),
            }),
        }
    }
}
