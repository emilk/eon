use crate::{
    Value,
    ast::{AstValue, CommentedValue},
    error::{Error, Result},
};

impl CommentedValue<'_> {
    pub fn try_into_value(self, source: &str) -> Result<Value> {
        match self.value {
            AstValue::Identifier(identifier) => match identifier.to_ascii_lowercase().as_str() {
                "null" | "nil" => Ok(Value::Null),
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err(Error::new_at(
                    source,
                    self.span,
                    "Unknown keyword. Expected 'null', 'true', or 'false'.",
                )),
            },
            AstValue::Number(string) => {
                crate::Number::try_parse(source, &string).map(Value::Number)
            }
            AstValue::String(_) => todo!(),
            AstValue::List(commented_list) => todo!(),
            AstValue::Object(commented_object) => todo!(),
        }
    }
}
