//! When we parse Con, we parse it into an AST with comments included.
//!
//! This is not always what we want, though,
//! so this module strips the comments.

use std::str::FromStr as _;

use crate::{Error, Result, Value};

use con_syntax::{CommentedChoice, Span, TokenTree, TreeValue, unescape_and_unquote};

impl Value {
    pub fn try_from_token_tree(con_source: &str, tt: &TokenTree<'_>) -> Result<Self> {
        Self::try_from_tree_value(con_source, tt.span, &tt.value)
    }

    pub fn try_from_tree_value(
        con_source: &str,
        span: Option<Span>,
        value: &TreeValue<'_>,
    ) -> Result<Self> {
        match value {
            TreeValue::Identifier(identifier) => match identifier.to_ascii_lowercase().as_str() {
                "null" | "nil" => Ok(Self::Null),
                "true" => Ok(Self::Bool(true)),
                "false" => Ok(Self::Bool(false)),
                _ => Err(Error::new(
                    con_source,
                    span,
                    "Unknown keyword. Expected 'null', 'true', or 'false'.",
                )),
            },
            TreeValue::Number(string) => {
                crate::Number::from_str(string)
                    .map(Self::Number)
                    .map_err(|err| {
                        Error::new(
                            con_source,
                            span,
                            format!("Failed to parse number: {err}. The string: {string:?}"),
                        )
                    })
            }
            TreeValue::QuotedString(escaped) => unescape_and_unquote(escaped)
                .map(Self::String)
                .map_err(|err| {
                    Error::new(
                        con_source,
                        span,
                        format!("Failed to unescape string: {err}. The string: {escaped}"),
                    )
                }),
            TreeValue::List(commented_list) => Ok(Self::List(
                commented_list
                    .values
                    .iter()
                    .map(|commented_value| Self::try_from_token_tree(con_source, commented_value))
                    .collect::<Result<_>>()?,
            )),
            TreeValue::Map(commented_map) => Ok(Self::Map(
                commented_map
                    .key_values
                    .iter()
                    .map(|commented_key_value| {
                        let key = match &commented_key_value.key.value {
                            TreeValue::Identifier(key) => key.to_string(),
                            _ => {
                                return Err(Error::new(
                                    con_source,
                                    span,
                                    "Expected an identifier for the map key.",
                                ));
                            }
                        };
                        // TODO: handle string keys
                        let value =
                            Self::try_from_token_tree(con_source, &commented_key_value.value)?;
                        Ok((key, value))
                    })
                    .collect::<Result<_>>()?,
            )),
            TreeValue::Choice(commented_choice) => {
                let CommentedChoice {
                    name_span,
                    quoted_name,
                    values,
                    closing_comments: _,
                } = commented_choice;
                let name = unescape_and_unquote(quoted_name).map_err(|err| {
                    Error::new(
                        con_source,
                        *name_span,
                        format!("Failed to unescape string: {err}. The string: {quoted_name}"),
                    )
                })?;
                let values = values
                    .iter()
                    .map(|commented_value| Self::try_from_token_tree(con_source, commented_value))
                    .collect::<Result<_>>()?;
                Ok(Self::Choice(crate::value::Choice { name, values }))
            }
        }
    }
}
