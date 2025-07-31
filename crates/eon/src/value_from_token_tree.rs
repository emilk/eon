//! When we parse Eon, we parse it into a [`TokenTree`] with comments included.
//!
//! This module parses the tokens, strips the comments,
//! and converts the [`TokenTree`] into a [`Value`].

use std::str::FromStr as _;

use crate::{Error, Map, Result, Value};

use eon_syntax::{Span, TokenKeyValue, TokenTree, TokenValue, TokenVariant, unescape_and_unquote};

impl Value {
    /// Try to parse a [`TokenTree`] into a [`Value`].
    ///
    /// You must provide the full Eon source string so that we can produce good error messages.
    pub fn try_from_token_tree(eon_source: &str, tt: &TokenTree<'_>) -> Result<Self> {
        Self::try_from_tree_value(eon_source, tt.span, &tt.value)
    }

    /// Try to parse a [`TokenValue`] into a [`Value`].
    ///
    /// You must provide the full Eon source string so that we can produce good error messages.
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
            TokenValue::Map(tt_map) => {
                let mut map = Map::with_capacity(tt_map.key_values.len());
                for TokenKeyValue { key: key_tt, value } in &tt_map.key_values {
                    let key = match &key_tt.value {
                        TokenValue::Identifier(key) => Self::String(key.to_string()),
                        _ => Self::try_from_token_tree(eon_source, key_tt)?,
                    };
                    let value = Self::try_from_token_tree(eon_source, value)?;
                    if map.insert(key, value).is_some() {
                        return Err(Error::new(eon_source, key_tt.span, "Duplicate key in map"));
                    }
                }
                Ok(Self::Map(map))
            }
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
