//! Metadata type for arbitrary key-value pairs.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Arbitrary key-value metadata for additional properties.
///
/// Keys and values are both strings. This is a simple wrapper around
/// a `HashMap` with serde support.
///
/// # Examples
///
/// ```
/// use agent_skills::Metadata;
///
/// let mut metadata = Metadata::new();
/// metadata.insert("author", "example-org");
/// metadata.insert("version", "1.0");
///
/// assert_eq!(metadata.get("author"), Some("example-org"));
/// assert_eq!(metadata.len(), 2);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Metadata(HashMap<String, String>);

impl Metadata {
    /// Creates a new empty metadata map.
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Creates metadata from an iterator of key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_skills::Metadata;
    ///
    /// let metadata = Metadata::from_pairs([
    ///     ("author", "test"),
    ///     ("version", "1.0"),
    /// ]);
    /// assert_eq!(metadata.len(), 2);
    /// ```
    pub fn from_pairs<K, V, I>(iter: I) -> Self
    where
        K: Into<String>,
        V: Into<String>,
        I: IntoIterator<Item = (K, V)>,
    {
        Self(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }

    /// Inserts a key-value pair.
    ///
    /// If the key already exists, the value is replaced.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    /// Gets a value by key.
    ///
    /// Returns `None` if the key doesn't exist.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// Returns `true` if the metadata is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Returns `true` if the metadata contains the specified key.
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Returns an iterator over the keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// Returns an iterator over the values.
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.0.values().map(String::as_str)
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (key, value) in &self.0 {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{key}: {value}")?;
            first = false;
        }
        write!(f, "}}")
    }
}

impl<K, V> FromIterator<(K, V)> for Metadata
where
    K: Into<String>,
    V: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_pairs(iter)
    }
}

impl IntoIterator for Metadata {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Metadata {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::hash_map::Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_metadata() {
        let metadata = Metadata::new();
        assert!(metadata.is_empty());
        assert_eq!(metadata.len(), 0);
    }

    #[test]
    fn insert_and_get_work() {
        let mut metadata = Metadata::new();
        metadata.insert("key", "value");
        assert_eq!(metadata.get("key"), Some("value"));
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let metadata = Metadata::new();
        assert_eq!(metadata.get("missing"), None);
    }

    #[test]
    fn from_iter_creates_populated_metadata() {
        let metadata = Metadata::from_pairs([("author", "test"), ("version", "1.0")]);
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata.get("author"), Some("test"));
        assert_eq!(metadata.get("version"), Some("1.0"));
    }

    #[test]
    fn contains_key_works() {
        let metadata = Metadata::from_pairs([("key", "value")]);
        assert!(metadata.contains_key("key"));
        assert!(!metadata.contains_key("missing"));
    }

    #[test]
    fn iter_works() {
        let metadata = Metadata::from_pairs([("a", "1"), ("b", "2")]);
        let pairs: Vec<_> = metadata.iter().collect();
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn keys_works() {
        let metadata = Metadata::from_pairs([("a", "1"), ("b", "2")]);
        let keys: Vec<_> = metadata.keys().collect();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn values_works() {
        let metadata = Metadata::from_pairs([("a", "1"), ("b", "2")]);
        let values: Vec<_> = metadata.values().collect();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn display_works() {
        let metadata = Metadata::from_pairs([("key", "value")]);
        let display = format!("{metadata}");
        assert!(display.contains("key"));
        assert!(display.contains("value"));
    }

    #[test]
    fn collect_works() {
        let pairs = vec![("a", "1"), ("b", "2")];
        let metadata: Metadata = pairs.into_iter().collect();
        assert_eq!(metadata.len(), 2);
    }

    #[test]
    fn into_iter_works() {
        let metadata = Metadata::from_pairs([("a", "1")]);
        let pairs: Vec<_> = metadata.into_iter().collect();
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn ref_into_iter_works() {
        let metadata = Metadata::from_pairs([("a", "1")]);
        let pairs: Vec<_> = (&metadata).into_iter().collect();
        assert_eq!(pairs.len(), 1);
    }
}
