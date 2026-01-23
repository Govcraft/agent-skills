//! Allowed tools type for pre-approved tool lists.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A space-delimited list of pre-approved tools.
///
/// This is an experimental feature per the Agent Skills specification.
///
/// # Examples
///
/// ```
/// use agent_skills::AllowedTools;
///
/// let tools = AllowedTools::new("Bash(git:*) Read Write");
/// assert_eq!(tools.as_slice().len(), 3);
/// assert_eq!(tools.as_slice()[0], "Bash(git:*)");
///
/// // Empty string creates empty tools
/// let empty = AllowedTools::new("");
/// assert!(empty.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AllowedTools(Vec<String>);

impl AllowedTools {
    /// Creates allowed tools from a space-delimited string.
    ///
    /// Empty strings or whitespace-only strings result in an empty list.
    #[must_use]
    pub fn new(tools: &str) -> Self {
        let tools: Vec<String> = tools
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        Self(tools)
    }

    /// Creates allowed tools from a vector of strings.
    #[must_use]
    pub const fn from_vec(tools: Vec<String>) -> Self {
        Self(tools)
    }

    /// Returns the tools as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Returns `true` if empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of tools.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the tools.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }

    /// Returns `true` if the list contains the specified tool.
    #[must_use]
    pub fn contains(&self, tool: &str) -> bool {
        self.0.iter().any(|t| t == tool)
    }
}

impl fmt::Display for AllowedTools {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

impl Serialize for AllowedTools {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as space-delimited string
        self.0.join(" ").serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AllowedTools {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(&s))
    }
}

impl IntoIterator for AllowedTools {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a AllowedTools {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<String> for AllowedTools {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parses_space_delimited_tools() {
        let tools = AllowedTools::new("Bash(git:*) Read Write");
        assert_eq!(tools.as_slice().len(), 3);
        assert_eq!(tools.as_slice()[0], "Bash(git:*)");
        assert_eq!(tools.as_slice()[1], "Read");
        assert_eq!(tools.as_slice()[2], "Write");
    }

    #[test]
    fn empty_string_creates_empty_tools() {
        let tools = AllowedTools::new("");
        assert!(tools.is_empty());
        assert_eq!(tools.len(), 0);
    }

    #[test]
    fn whitespace_only_creates_empty_tools() {
        let tools = AllowedTools::new("   \t\n  ");
        assert!(tools.is_empty());
    }

    #[test]
    fn handles_multiple_spaces() {
        let tools = AllowedTools::new("Read   Write    Execute");
        assert_eq!(tools.len(), 3);
    }

    #[test]
    fn from_vec_works() {
        let tools = AllowedTools::from_vec(vec!["Read".to_string(), "Write".to_string()]);
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn iter_works() {
        let tools = AllowedTools::new("Read Write");
        let items: Vec<_> = tools.iter().collect();
        assert_eq!(items, vec!["Read", "Write"]);
    }

    #[test]
    fn contains_works() {
        let tools = AllowedTools::new("Read Write");
        assert!(tools.contains("Read"));
        assert!(tools.contains("Write"));
        assert!(!tools.contains("Execute"));
    }

    #[test]
    fn display_works() {
        let tools = AllowedTools::new("Read Write");
        assert_eq!(format!("{tools}"), "Read Write");
    }

    #[test]
    fn into_iter_works() {
        let tools = AllowedTools::new("Read Write");
        let items: Vec<String> = tools.into_iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn ref_into_iter_works() {
        let tools = AllowedTools::new("Read Write");
        let items: Vec<_> = (&tools).into_iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn collect_works() {
        let items = vec!["Read".to_string(), "Write".to_string()];
        let tools: AllowedTools = items.into_iter().collect();
        assert_eq!(tools.len(), 2);
    }
}
