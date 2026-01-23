//! Skill description type with validation.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Error returned when a skill description is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillDescriptionError {
    /// The description is empty or whitespace-only.
    Empty,
    /// The description exceeds the maximum length.
    TooLong {
        /// The actual length of the description.
        length: usize,
        /// The maximum allowed length.
        max: usize,
    },
}

impl fmt::Display for SkillDescriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "skill description cannot be empty or whitespace-only"),
            Self::TooLong { length, max } => {
                write!(
                    f,
                    "skill description is {length} characters; maximum is {max}"
                )
            }
        }
    }
}

impl std::error::Error for SkillDescriptionError {}

/// A validated skill description.
///
/// # Constraints
///
/// - Must be 1-1024 characters
/// - Must not be empty or whitespace-only
///
/// # Examples
///
/// ```
/// use agent_skills::SkillDescription;
///
/// let desc = SkillDescription::new("Extracts text from PDF files.").unwrap();
/// assert!(!desc.as_str().is_empty());
///
/// // Invalid: empty
/// assert!(SkillDescription::new("").is_err());
///
/// // Invalid: whitespace only
/// assert!(SkillDescription::new("   ").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillDescription(String);

impl SkillDescription {
    /// Maximum length for a skill description.
    pub const MAX_LENGTH: usize = 1024;

    /// Creates a new skill description after validation.
    ///
    /// # Errors
    ///
    /// Returns `SkillDescriptionError` if the description is invalid.
    pub fn new(description: impl Into<String>) -> Result<Self, SkillDescriptionError> {
        let description = description.into();
        validate_skill_description(&description)?;
        Ok(Self(description))
    }

    /// Returns the description as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SkillDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SkillDescription {
    type Err = SkillDescriptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for SkillDescription {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for SkillDescription {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SkillDescription {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

/// Validates a skill description string.
fn validate_skill_description(description: &str) -> Result<(), SkillDescriptionError> {
    // Check empty or whitespace-only
    if description.trim().is_empty() {
        return Err(SkillDescriptionError::Empty);
    }

    // Check length
    if description.len() > SkillDescription::MAX_LENGTH {
        return Err(SkillDescriptionError::TooLong {
            length: description.len(),
            max: SkillDescription::MAX_LENGTH,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_description_is_accepted() {
        let desc = SkillDescription::new("Extracts text from PDF files.");
        assert!(desc.is_ok());
    }

    #[test]
    fn description_at_max_length_is_accepted() {
        let desc = "a".repeat(1024);
        assert!(SkillDescription::new(desc).is_ok());
    }

    #[test]
    fn empty_description_is_rejected() {
        let result = SkillDescription::new("");
        assert_eq!(result, Err(SkillDescriptionError::Empty));
    }

    #[test]
    fn whitespace_only_description_is_rejected() {
        let result = SkillDescription::new("   \n\t  ");
        assert_eq!(result, Err(SkillDescriptionError::Empty));
    }

    #[test]
    fn description_exceeding_max_length_is_rejected() {
        let long_desc = "a".repeat(1025);
        let result = SkillDescription::new(long_desc);
        assert!(matches!(
            result,
            Err(SkillDescriptionError::TooLong {
                length: 1025,
                max: 1024
            })
        ));
    }

    #[test]
    fn description_with_leading_trailing_whitespace_is_accepted() {
        // We allow leading/trailing whitespace as long as there's content
        let desc = SkillDescription::new("  Some description  ");
        assert!(desc.is_ok());
    }

    #[test]
    fn display_returns_inner_string() {
        let desc = SkillDescription::new("My description").unwrap();
        assert_eq!(format!("{desc}"), "My description");
    }

    #[test]
    fn from_str_works() {
        let desc: SkillDescription = "My description".parse().unwrap();
        assert_eq!(desc.as_str(), "My description");
    }

    #[test]
    fn as_ref_works() {
        let desc = SkillDescription::new("My description").unwrap();
        let s: &str = desc.as_ref();
        assert_eq!(s, "My description");
    }

    #[test]
    fn error_display_is_helpful() {
        let err = SkillDescriptionError::TooLong {
            length: 2000,
            max: 1024,
        };
        let msg = err.to_string();
        assert!(msg.contains("2000"));
        assert!(msg.contains("1024"));
    }
}
