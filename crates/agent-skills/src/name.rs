//! Skill name type with validation.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Error returned when a skill name is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillNameError {
    /// The name is empty.
    Empty,
    /// The name exceeds the maximum length.
    TooLong {
        /// The actual length of the name.
        length: usize,
        /// The maximum allowed length.
        max: usize,
    },
    /// The name contains an invalid character.
    InvalidCharacter {
        /// The invalid character.
        char: char,
        /// The position of the invalid character.
        position: usize,
    },
    /// The name starts with a hyphen.
    StartsWithHyphen,
    /// The name ends with a hyphen.
    EndsWithHyphen,
    /// The name contains consecutive hyphens.
    ConsecutiveHyphens {
        /// The position of the first consecutive hyphen.
        position: usize,
    },
}

impl fmt::Display for SkillNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "skill name cannot be empty"),
            Self::TooLong { length, max } => {
                write!(f, "skill name is {length} characters; maximum is {max}")
            }
            Self::InvalidCharacter { char, position } => {
                write!(
                    f,
                    "invalid character '{char}' at position {position}; only lowercase alphanumeric and hyphens allowed"
                )
            }
            Self::StartsWithHyphen => write!(f, "skill name cannot start with a hyphen"),
            Self::EndsWithHyphen => write!(f, "skill name cannot end with a hyphen"),
            Self::ConsecutiveHyphens { position } => {
                write!(
                    f,
                    "consecutive hyphens at position {position}; use single hyphens only"
                )
            }
        }
    }
}

impl std::error::Error for SkillNameError {}

/// A validated skill name.
///
/// # Constraints
///
/// - Must be 1-64 Unicode scalar values
/// - May only contain Unicode alphanumeric characters unchanged by lowercasing,
///   and ASCII hyphens (`-`)
/// - Must not start or end with a hyphen
/// - Must not contain consecutive hyphens (`--`)
///
/// Names are preserved exactly, without Unicode normalization. Directory names
/// must match this exact representation. Uncased letters and Unicode numbers
/// are accepted; combining marks are not alphanumeric and are rejected.
///
/// # Examples
///
/// ```
/// use agent_skills::SkillName;
///
/// let name = SkillName::new("pdf-processing").unwrap();
/// assert_eq!(name.as_str(), "pdf-processing");
///
/// // Invalid: uppercase
/// assert!(SkillName::new("PDF-Processing").is_err());
///
/// // Invalid: consecutive hyphens
/// assert!(SkillName::new("pdf--processing").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillName(String);

impl SkillName {
    /// Maximum number of Unicode scalar values in a skill name.
    pub const MAX_LENGTH: usize = 64;

    /// Creates a new skill name after validation.
    ///
    /// # Errors
    ///
    /// Returns `SkillNameError` if the name is invalid.
    pub fn new(name: impl Into<String>) -> Result<Self, SkillNameError> {
        let name = name.into();
        validate_skill_name(&name)?;
        Ok(Self(name))
    }

    /// Returns the skill name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SkillName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SkillName {
    type Err = SkillNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for SkillName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for SkillName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SkillName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

/// Validates a skill name string.
fn validate_skill_name(name: &str) -> Result<(), SkillNameError> {
    // Check empty
    if name.is_empty() {
        return Err(SkillNameError::Empty);
    }

    // Check length
    let length = name.chars().count();
    if length > SkillName::MAX_LENGTH {
        return Err(SkillNameError::TooLong {
            length,
            max: SkillName::MAX_LENGTH,
        });
    }

    validate_skill_name_syntax(name)
}

/// Validates name syntax without imposing a length limit.
pub fn validate_skill_name_syntax(name: &str) -> Result<(), SkillNameError> {
    if name.is_empty() {
        return Err(SkillNameError::Empty);
    }

    // Check starts with hyphen
    if name.starts_with('-') {
        return Err(SkillNameError::StartsWithHyphen);
    }

    // Check ends with hyphen
    if name.ends_with('-') {
        return Err(SkillNameError::EndsWithHyphen);
    }

    // Check each character and consecutive hyphens
    let mut prev_was_hyphen = false;
    for (position, char) in name.chars().enumerate() {
        if char == '-' {
            if prev_was_hyphen {
                return Err(SkillNameError::ConsecutiveHyphens { position });
            }
            prev_was_hyphen = true;
        } else if char.is_alphanumeric() && char.to_lowercase().eq(std::iter::once(char)) {
            prev_was_hyphen = false;
        } else {
            return Err(SkillNameError::InvalidCharacter { char, position });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_name_is_accepted() -> Result<(), SkillNameError> {
        let name = SkillName::new("pdf-processing")?;
        assert_eq!(name.as_str(), "pdf-processing");
        Ok(())
    }

    #[test]
    fn simple_name_is_accepted() {
        let name = SkillName::new("skill");
        assert!(name.is_ok());
    }

    #[test]
    fn name_with_numbers_is_accepted() {
        let name = SkillName::new("tool2go");
        assert!(name.is_ok());
    }

    #[test]
    fn name_at_max_length_is_accepted() {
        let name = "a".repeat(64);
        assert!(SkillName::new(name).is_ok());
    }

    #[test]
    fn empty_name_is_rejected() {
        let result = SkillName::new("");
        assert_eq!(result, Err(SkillNameError::Empty));
    }

    #[test]
    fn name_exceeding_max_length_is_rejected() {
        let long_name = "a".repeat(65);
        let result = SkillName::new(long_name);
        assert!(matches!(
            result,
            Err(SkillNameError::TooLong {
                length: 65,
                max: 64
            })
        ));
    }

    #[test]
    fn uppercase_characters_are_rejected() {
        let result = SkillName::new("PDF-Processing");
        assert!(matches!(
            result,
            Err(SkillNameError::InvalidCharacter {
                char: 'P',
                position: 0
            })
        ));
    }

    #[test]
    fn uppercase_in_middle_is_rejected() {
        let result = SkillName::new("pdfProcessing");
        assert!(matches!(
            result,
            Err(SkillNameError::InvalidCharacter {
                char: 'P',
                position: 3
            })
        ));
    }

    #[test]
    fn name_starting_with_hyphen_is_rejected() {
        let result = SkillName::new("-pdf");
        assert_eq!(result, Err(SkillNameError::StartsWithHyphen));
    }

    #[test]
    fn name_ending_with_hyphen_is_rejected() {
        let result = SkillName::new("pdf-");
        assert_eq!(result, Err(SkillNameError::EndsWithHyphen));
    }

    #[test]
    fn consecutive_hyphens_are_rejected() {
        let result = SkillName::new("pdf--processing");
        assert!(matches!(
            result,
            Err(SkillNameError::ConsecutiveHyphens { position: 4 })
        ));
    }

    #[test]
    fn underscore_is_rejected() {
        let result = SkillName::new("pdf_processing");
        assert!(matches!(
            result,
            Err(SkillNameError::InvalidCharacter {
                char: '_',
                position: 3
            })
        ));
    }

    #[test]
    fn space_is_rejected() {
        let result = SkillName::new("pdf processing");
        assert!(matches!(
            result,
            Err(SkillNameError::InvalidCharacter {
                char: ' ',
                position: 3
            })
        ));
    }

    #[test]
    fn display_returns_inner_string() -> Result<(), SkillNameError> {
        let name = SkillName::new("my-skill")?;
        assert_eq!(format!("{name}"), "my-skill");
        Ok(())
    }

    #[test]
    fn from_str_works() -> Result<(), SkillNameError> {
        let name: SkillName = "my-skill".parse()?;
        assert_eq!(name.as_str(), "my-skill");
        Ok(())
    }

    #[test]
    fn as_ref_works() -> Result<(), SkillNameError> {
        let name = SkillName::new("my-skill")?;
        let s: &str = name.as_ref();
        assert_eq!(s, "my-skill");
        Ok(())
    }

    #[test]
    fn error_display_is_helpful() {
        let err = SkillNameError::InvalidCharacter {
            char: 'X',
            position: 5,
        };
        let msg = err.to_string();
        assert!(msg.contains("'X'"));
        assert!(msg.contains("position 5"));
        assert!(msg.contains("lowercase alphanumeric"));
    }

    #[test]
    fn unicode_names_are_preserved() -> Result<(), SkillNameError> {
        for input in ["café", "技能", "инструмент", "tool-٢", "𐐨"] {
            assert_eq!(SkillName::new(input)?.as_str(), input);
        }
        Ok(())
    }

    #[test]
    fn unicode_length_counts_scalar_values() {
        for character in ["é", "技", "𐐨"] {
            assert!(SkillName::new(character.repeat(64)).is_ok());
            assert_eq!(
                SkillName::new(character.repeat(65)),
                Err(SkillNameError::TooLong {
                    length: 65,
                    max: 64
                })
            );
        }
    }

    #[test]
    fn unicode_invalid_characters_report_scalar_positions() {
        for character in ['É', 'ǅ', '𐐀', '＿', '💡', '\u{301}'] {
            assert_eq!(
                SkillName::new(format!("é{character}")),
                Err(SkillNameError::InvalidCharacter {
                    char: character,
                    position: 1
                })
            );
        }
        assert_eq!(
            SkillName::new("é--技"),
            Err(SkillNameError::ConsecutiveHyphens { position: 2 })
        );
    }

    #[test]
    fn unicode_names_round_trip_through_serde() -> Result<(), Box<dyn std::error::Error>> {
        let name = SkillName::new("café-技能")?;
        let serialized = serde_yml::to_string(&name)?;
        let deserialized: SkillName = serde_yml::from_str(&serialized)?;
        assert_eq!(deserialized, name);
        Ok(())
    }
}
