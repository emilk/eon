use crate::{Number, Value};

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::Number(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::Map(o) => o.serialize(serializer),
            Value::List(a) => a.serialize(serializer),
            Value::Null => serializer.serialize_none(),
        }
    }
}

impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(n) = self.as_u64() {
            n.serialize(serializer)
        } else if let Some(n) = self.as_i64() {
            n.serialize(serializer)
        } else if let Some(n) = self.as_f64() {
            n.serialize(serializer)
        } else if let Some(n) = self.as_i128() {
            n.serialize(serializer)
        } else if let Some(n) = self.as_u128() {
            n.serialize(serializer)
        } else {
            return Err(S::Error::custom(format!("Invalid numbner: {self}")));
        }
    }
}
