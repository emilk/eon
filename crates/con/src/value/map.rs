use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

use crate::Value;

/// Maps values to values.
///
/// Usually maps strings to values, but can map any value to any value.
///
/// The iteration order is guaranteed to be the same as the insertion order.
/// However, equality and the hash is indepenent of the insertion order.
#[derive(Default, Debug, Clone, Eq)]
pub struct Map {
    map: indexmap::IndexMap<Value, Value>,

    // In order to implement `Hash` efficiently,
    // we keep a running xor of the hash of all the keys and values.
    hash_of_keys: u64,

    hash_of_values: u64,
}

impl Map {
    /// Creates a new empty `Map`.
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates a new empty `Map` with the given capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: indexmap::IndexMap::with_capacity(capacity),
            hash_of_keys: 0,
            hash_of_values: 0,
        }
    }

    #[inline]
    pub fn insert(&mut self, key: Value, value: Value) -> Option<Value> {
        let key_hash = hash_of(&key);
        let value_hash = hash_of(&value);
        self.hash_of_keys ^= key_hash; // Using XOR guarantees that it's order-independent
        self.hash_of_values ^= value_hash; // Using XOR guarantees that it's order-independent
        self.map.insert(key, value)
    }
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.hash_of_keys == other.hash_of_keys
            && self.hash_of_values == other.hash_of_values
            && self.map == other.map
    }
}

impl Hash for Map {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            hash_of_keys,
            hash_of_values,
            map,
        } = self;
        hash_of_keys.hash(state);
        hash_of_values.hash(state);
        map.len().hash(state);
    }
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

impl FromIterator<(String, Value)> for Map {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut map = Self::with_capacity(iter.size_hint().0);
        for (key, value) in iter {
            map.insert(Value::String(key), value);
        }
        map
    }
}

impl FromIterator<(Value, Value)> for Map {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (Value, Value)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut map = Self::with_capacity(iter.size_hint().0);
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<'a> IntoIterator for &'a Map {
    type Item = (&'a Value, &'a Value);
    type IntoIter = indexmap::map::Iter<'a, Value, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut Map {
    type Item = (&'a Value, &'a mut Value);
    type IntoIter = indexmap::map::IterMut<'a, Value, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl IntoIterator for Map {
    type Item = (Value, Value);
    type IntoIter = indexmap::map::IntoIter<Value, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl Deref for Map {
    type Target = indexmap::IndexMap<Value, Value>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[test]
fn test_map() {
    let map_a = Map::from_iter([
        (Value::String("a".into()), Value::Number(1.into())),
        (Value::String("b".into()), Value::Number(2.into())),
        (Value::String("c".into()), Value::Number(3.into())),
        (Value::String("d".into()), Value::Number(4.into())),
    ]);
    let map_b = Map::from_iter([
        (Value::String("d".into()), Value::Number(4.into())),
        (Value::String("c".into()), Value::Number(3.into())),
        (Value::String("b".into()), Value::Number(2.into())),
        (Value::String("a".into()), Value::Number(1.into())),
    ]);

    assert_eq!(map_a, map_b);
    assert_eq!(hash_of(&map_a), hash_of(&map_b));
}
