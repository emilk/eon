use std::ops::{Deref, DerefMut};

use crate::Value;

/// Maps strings to values, i.e. like a `struct`.
#[derive(Default)]
pub struct Map(indexmap::IndexMap<String, Value>); // TODO: Value to Value

impl Map {
    /// Creates a new empty `Map`.
    #[inline]
    pub fn new() -> Self {
        Self(indexmap::IndexMap::new())
    }

    /// Creates a new empty `Map` with the given capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(indexmap::IndexMap::with_capacity(capacity))
    }
}

impl FromIterator<(String, Value)> for Map {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Self(indexmap::IndexMap::from_iter(iter))
    }
}

impl<'a> IntoIterator for &'a Map {
    type Item = (&'a String, &'a Value);
    type IntoIter = indexmap::map::Iter<'a, String, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Map {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = indexmap::map::IterMut<'a, String, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl IntoIterator for Map {
    type Item = (String, Value);
    type IntoIter = indexmap::map::IntoIter<String, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Map {
    type Target = indexmap::IndexMap<String, Value>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Map {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
