//! Compatibility string type with validation.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Error returned when a compatibility string is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityError {
    /// The compatibility string is empty.
    Empty,
    /// The compatibility string exceeds the maximum length.
    TooLong {
        /// The actual length.
        length: usize,
        /// The maximum allowed length.
        max: usize,
    },
}

impl fmt::Display for CompatibilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "compatibility string cannot be empty"),
            Self::TooLong { length, max } => {
                write!(
                    f,
                    "compatibility string is {length} characters; maximum is {max}"
                )
            }
        }
    }
}

impl std::error::Error for CompatibilityError {}

/// An optional compatibility string describing environment requirements.
///
/// # Constraints
///
/// - Must be 1-500 characters if provided
///
/// # Examples
///
/// ```
/// use agent_skills::Compatibility;
///
/// let compat = Compatibility::new("Requires git, docker, jq").unwrap();
/// assert_eq!(compat.as_str(), "Requires git, docker, jq");
///
/// // Invalid: empty
/// assert!(Compatibility::new("").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Compatibility(String);

impl Compatibility {
    /// Maximum length for a compatibility string.
    pub const MAX_LENGTH: usize = 500;

    /// Creates a new compatibility string after validation.
    ///
    /// # Errors
    ///
    /// Returns `CompatibilityError` if the string is invalid.
    pub fn new(compat: impl Into<String>) -> Result<Self, CompatibilityError> {
        let compat = compat.into();
        validate_compatibility(&compat)?;
        Ok(Self(compat))
    }

    /// Returns the compatibility string as a slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Compatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Compatibility {
    type Err = CompatibilityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for Compatibility {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for Compatibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Compatibility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

/// Validates a compatibility string.
const fn validate_compatibility(compat: &str) -> Result<(), CompatibilityError> {
    // Check empty
    if compat.is_empty() {
        return Err(CompatibilityError::Empty);
    }

    // Check length
    if compat.len() > Compatibility::MAX_LENGTH {
        return Err(CompatibilityError::TooLong {
            length: compat.len(),
            max: Compatibility::MAX_LENGTH,
        });
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn valid_compatibility_is_accepted() {
        let compat = Compatibility::new("Requires git, docker, jq");
        assert!(compat.is_ok());
    }

    #[test]
    fn compatibility_at_max_length_is_accepted() {
        let compat = "a".repeat(500);
        assert!(Compatibility::new(compat).is_ok());
    }

    #[test]
    fn empty_compatibility_is_rejected() {
        let result = Compatibility::new("");
        assert_eq!(result, Err(CompatibilityError::Empty));
    }

    #[test]
    fn compatibility_exceeding_max_length_is_rejected() {
        let long_compat = "a".repeat(501);
        let result = Compatibility::new(long_compat);
        assert!(matches!(
            result,
            Err(CompatibilityError::TooLong {
                length: 501,
                max: 500
            })
        ));
    }

    #[test]
    fn display_returns_inner_string() {
        let compat = Compatibility::new("Designed for Claude Code").unwrap();
        assert_eq!(format!("{compat}"), "Designed for Claude Code");
    }

    #[test]
    fn from_str_works() {
        let compat: Compatibility = "Requires docker".parse().unwrap();
        assert_eq!(compat.as_str(), "Requires docker");
    }

    #[test]
    fn as_ref_works() {
        let compat = Compatibility::new("Test").unwrap();
        let s: &str = compat.as_ref();
        assert_eq!(s, "Test");
    }
}
