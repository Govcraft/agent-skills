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
        /// The actual number of Unicode scalar values.
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
/// - Must be 1-500 Unicode scalar values if provided
///
/// # Examples
///
/// ```
/// use agent_skills::Compatibility;
///
/// let compat = Compatibility::new("Requires git, docker, jq")?;
/// assert_eq!(compat.as_str(), "Requires git, docker, jq");
///
/// // Invalid: empty
/// assert!(Compatibility::new("").is_err());
/// # Ok::<(), agent_skills::CompatibilityError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Compatibility(String);

impl Compatibility {
    /// Maximum number of Unicode scalar values in a compatibility string.
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
fn validate_compatibility(compat: &str) -> Result<(), CompatibilityError> {
    // Check empty
    if compat.is_empty() {
        return Err(CompatibilityError::Empty);
    }

    // Check length
    let length = compat.chars().count();
    if length > Compatibility::MAX_LENGTH {
        return Err(CompatibilityError::TooLong {
            length,
            max: Compatibility::MAX_LENGTH,
        });
    }

    Ok(())
}

#[cfg(test)]
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
    fn display_returns_inner_string() -> Result<(), CompatibilityError> {
        let compat = Compatibility::new("Designed for Claude Code")?;
        assert_eq!(format!("{compat}"), "Designed for Claude Code");
        Ok(())
    }

    #[test]
    fn from_str_works() -> Result<(), CompatibilityError> {
        let compat: Compatibility = "Requires docker".parse()?;
        assert_eq!(compat.as_str(), "Requires docker");
        Ok(())
    }

    #[test]
    fn as_ref_works() -> Result<(), CompatibilityError> {
        let compat = Compatibility::new("Test")?;
        let s: &str = compat.as_ref();
        assert_eq!(s, "Test");
        Ok(())
    }
    #[test]
    fn multibyte_text_obeys_character_limit() {
        for character in ["é", "界", "🦀"] {
            assert!(Compatibility::new(character.repeat(500)).is_ok());
            assert_eq!(
                Compatibility::new(character.repeat(501)),
                Err(CompatibilityError::TooLong {
                    length: 501,
                    max: 500
                })
            );
        }
    }

    #[test]
    fn previously_rejected_accented_text_is_accepted() {
        assert!(Compatibility::new("é".repeat(300)).is_ok());
    }

    #[test]
    fn mixed_width_text_obeys_character_limit() {
        let text = format!("{}é界🦀", "a".repeat(497));
        assert!(Compatibility::new(&text).is_ok());
        assert_eq!(
            Compatibility::new(format!("{text}é")),
            Err(CompatibilityError::TooLong {
                length: 501,
                max: 500
            })
        );
    }

    #[test]
    fn combining_sequences_count_as_multiple_scalar_values() {
        let text = "e\u{301}".repeat(250);
        assert!(Compatibility::new(&text).is_ok());
        assert_eq!(
            Compatibility::new(format!("{text}e\u{301}")),
            Err(CompatibilityError::TooLong {
                length: 502,
                max: 500
            })
        );
    }
}
