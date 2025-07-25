use serde::ser::{Error as _, SerializeMap as _};

use crate::{Map, Number, Value};

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::Number(n) => n.serialize(serializer),
            Self::String(s) => serializer.serialize_str(s),
            Self::Map(map) => map.serialize(serializer),
            Self::List(a) => a.serialize(serializer),
            Self::Choice(choice) => {
                // TODO: should we even implement Serialize for Value?
                Err(S::Error::custom(format!(
                    "Cannot serialize Choice directly (not supported by serde): {choice:?}"
                )))
            }
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

impl serde::Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer
            .serialize_map(Some(self.len()))
            .and_then(|mut map| {
                for (key, value) in self {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            })
    }
}
