//! The formatter uses the [`CommentedValue`] type,
//! so in order to print a [`Value`], we need a way to convert it to [`CommentedValue`].

use crate::{
    Value,
    ast::{AstValue, CommentedValue},
};

// impl From<Value> for CommentedValue<'static> {
//     fn from(value: Value) -> Self {
//         CommentedValue {
//             span: Default::default(),
//             prefix_comments: vec![],
//             value: AstValue::from(value),
//             suffix_comment: None,
//         }
//     }
// }

// impl From<Value> for AstValue<'static> {
//     fn from(value: Value) -> Self {
//         match value {
//             Value::Null => AstValue::Keyword("null".into()),
//             Value::Bool(true) => AstValue::Keyword("true".into()),
//             Value::Bool(false) => AstValue::Keyword("false".into()),
//             Value::Number(number) => AstValue::Number(number.to_string()),
//             Value::String(string) => AstValue::String(string.into()),
//             Value::List(list) => AstValue::List(list.into_iter().map(Into::into).collect()),
//             Value::Object(object) => {
//                 AstValue::Object(object.into_iter().map(|(k, v)| (k, v.into())).collect())
//             }
//         }
//     }
// }
