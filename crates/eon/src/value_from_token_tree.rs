//! When we parse Eon, we parse it into a [`TokenTree`] with comments included.
//!
//! This module parses the tokens, strips the comments,
//! and converts the [`TokenTree`] into a [`Value`].

use std::str::FromStr as _;

use crate::{Error, Result, Value};

use eon_syntax::{Span, TokenKeyValue, TokenTree, TokenValue, TokenVariant, unescape_and_unquote};

impl Value {
    pub fn try_from_token_tree(eon_source: &str, tt: &TokenTree<'_>) -> Result<Self> {
        Self::try_from_tree_value(eon_source, tt.span, &tt.value)
    }

    pub fn try_from_tree_value(
        eon_source: &str,
        span: Option<Span>,
        value: &TokenValue<'_>,
    ) -> Result<Self> {
        match value {
            TokenValue::Identifier(identifier) => match identifier.as_ref() {
                "null" => Ok(Self::Null),
                "true" => Ok(Self::Bool(true)),
                "false" => Ok(Self::Bool(false)),
                _ => {
                    let suggestion = match identifier.to_lowercase().as_str() {
                        "inf" => Some("+inf or -inf"),
                        "nan" => Some("+nan"),
                        "false" => Some("false"),
                        "true" => Some("true"),
                        "nil" | "null" | "none" => Some("null"),
                        _ => None,
                    };

                    let msg = if let Some(suggestion) = suggestion {
                        format!("Unknown keyword {identifier:?}. Did you mean: {suggestion}?")
                    } else {
                        format!(
                            "Unknown keyword {identifier:?}. Expected 'null', 'true', or 'false'."
                        )
                    };

                    Err(Error::new(eon_source, span, msg))
                }
            },
            TokenValue::Number(string) => crate::Number::from_str(string)
                .map(Self::Number)
                .map_err(|err| {
                    Error::new(
                        eon_source,
                        span,
                        format!("Failed to parse number: {err}. The string: {string:?}"),
                    )
                }),
            TokenValue::QuotedString(escaped) => unescape_and_unquote(escaped)
                .map(Self::String)
                .map_err(|err| {
                    Error::new(
                        eon_source,
                        span,
                        format!("Failed to unescape string: {err}. The string: {escaped}"),
                    )
                }),
            TokenValue::List(list) => Ok(Self::List(
                list.values
                    .iter()
                    .map(|value| Self::try_from_token_tree(eon_source, value))
                    .collect::<Result<_>>()?,
            )),
            TokenValue::Map(maps) => Ok(Self::Map(
                maps.key_values
                    .iter()
                    .map(|TokenKeyValue { key, value }| {
                        let key = match &key.value {
                            TokenValue::Identifier(key) => Self::String(key.to_string()),
                            _ => Self::try_from_token_tree(eon_source, key)?,
                        };
                        let value = Self::try_from_token_tree(eon_source, value)?;
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
                        eon_source,
                        *name_span,
                        format!("Failed to unescape string: {err}. The string: {quoted_name}"),
                    )
                })?;
                let values = values
                    .iter()
                    .map(|token_tree| Self::try_from_token_tree(eon_source, token_tree))
                    .collect::<Result<_>>()?;
                Ok(Self::new_variant(name, values))
            }
        }
    }
}
