//! When we parse Con, we parse it into a [`TokenTree`] with comments included.
//!
//! This module parses the tokens, strips the comments,
//! and converts the [`TokenTree`] into a [`Value`].

use std::str::FromStr as _;

use crate::{Error, Result, Value};

use con_syntax::{Span, TokenKeyValue, TokenTree, TokenValue, TokenVariant, unescape_and_unquote};

impl Value {
    pub fn try_from_token_tree(con_source: &str, tt: &TokenTree<'_>) -> Result<Self> {
        Self::try_from_tree_value(con_source, tt.span, &tt.value)
    }

    pub fn try_from_tree_value(
        con_source: &str,
        span: Option<Span>,
        value: &TokenValue<'_>,
    ) -> Result<Self> {
        match value {
            TokenValue::Identifier(identifier) => match identifier.to_ascii_lowercase().as_str() {
                "null" | "nil" => Ok(Self::Null),
                "true" => Ok(Self::Bool(true)),
                "false" => Ok(Self::Bool(false)),
                _ => Err(Error::new(
                    con_source,
                    span,
                    "Unknown keyword. Expected 'null', 'true', or 'false'.",
                )),
            },
            TokenValue::Number(string) => crate::Number::from_str(string)
                .map(Self::Number)
                .map_err(|err| {
                    Error::new(
                        con_source,
                        span,
                        format!("Failed to parse number: {err}. The string: {string:?}"),
                    )
                }),
            TokenValue::QuotedString(escaped) => unescape_and_unquote(escaped)
                .map(Self::String)
                .map_err(|err| {
                    Error::new(
                        con_source,
                        span,
                        format!("Failed to unescape string: {err}. The string: {escaped}"),
                    )
                }),
            TokenValue::List(list) => Ok(Self::List(
                list.values
                    .iter()
                    .map(|value| Self::try_from_token_tree(con_source, value))
                    .collect::<Result<_>>()?,
            )),
            TokenValue::Map(maps) => Ok(Self::Map(
                maps.key_values
                    .iter()
                    .map(|TokenKeyValue { key, value }| {
                        let key = match &key.value {
                            TokenValue::Identifier(key) => key.to_string(),
                            // TODO: handle string keys, and Variant keys.
                            _ => {
                                return Err(Error::new(
                                    con_source,
                                    span,
                                    "Expected an identifier for the map key.",
                                ));
                            }
                        };
                        let value = Self::try_from_token_tree(con_source, value)?;
                        Ok((key, value))
                    })
                    .collect::<Result<_>>()?,
            )),
            TokenValue::Variant(variant) => {
                let TokenVariant {
                    name_span,
                    quoted_name,
                    values,
                    closing_comments: _,
                } = variant;
                let name = unescape_and_unquote(quoted_name).map_err(|err| {
                    Error::new(
                        con_source,
                        *name_span,
                        format!("Failed to unescape string: {err}. The string: {quoted_name}"),
                    )
                })?;
                let values = values
                    .iter()
                    .map(|token_tree| Self::try_from_token_tree(con_source, token_tree))
                    .collect::<Result<_>>()?;
                Ok(Self::Variant(crate::value::Variant { name, values }))
            }
        }
    }
}
