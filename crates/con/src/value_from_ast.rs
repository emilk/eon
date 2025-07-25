//! When we parse Con, we parse it into an AST with comments included.
//!
//! This is not always what we want, though,
//! so this module strips the comments.

use crate::{
    Value,
    ast::{AstValue, CommentedValue},
    error::{Error, Result},
    span::Span,
};

impl CommentedValue<'_> {
    pub fn try_into_value(self, source: &str) -> Result<Value> {
        self.value.try_into_value(source, self.span)
    }
}

impl AstValue<'_> {
    pub fn try_into_value(self, source: &str, span: Span) -> Result<Value> {
        match self {
            AstValue::Identifier(identifier) => match identifier.to_ascii_lowercase().as_str() {
                "null" | "nil" => Ok(Value::Null),
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err(Error::new_at(
                    source,
                    span,
                    "Unknown keyword. Expected 'null', 'true', or 'false'.",
                )),
            },
            AstValue::Number(string) => crate::Number::try_parse(&string)
                .map(Value::Number)
                .map_err(|err| {
                    Error::new_at(
                        source,
                        span,
                        format!("Failed to parse number: {err}. The string: {string:?}"),
                    )
                }),
            AstValue::QuotedString(escaped) => match snailquote::unescape(&escaped) {
                Ok(unescaped) => Ok(Value::String(unescaped)),
                Err(err) => Err(Error::new_at(
                    source,
                    span,
                    format!("Failed to unescape string: {err}. The string: {escaped}"),
                )),
            },
            AstValue::List(commented_list) => Ok(Value::List(
                commented_list
                    .values
                    .into_iter()
                    .map(|commented_value| commented_value.try_into_value(source))
                    .collect::<Result<_>>()?,
            )),
            AstValue::Map(commented_map) => Ok(Value::Map(
                commented_map
                    .key_values
                    .into_iter()
                    .map(|commented_key_value| {
                        let key = match commented_key_value.key.value {
                            AstValue::Identifier(key) => key.into_owned(),
                            _ => {
                                return Err(Error::new_at(
                                    source,
                                    span,
                                    "Expected an identifier for the map key.",
                                ));
                            }
                        };
                        // TODO: handle string keys
                        let value = commented_key_value.value.try_into_value(source)?;
                        Ok((key, value))
                    })
                    .collect::<Result<_>>()?,
            )),
        }
    }
}
