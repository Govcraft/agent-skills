//! Error types for parsing and loading skills.

use std::fmt;

use crate::compatibility::CompatibilityError;
use crate::description::SkillDescriptionError;
use crate::name::SkillNameError;

/// Error returned when parsing a SKILL.md file fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The content doesn't start with the frontmatter delimiter.
    MissingFrontmatter,
    /// The frontmatter is not properly terminated.
    UnterminatedFrontmatter,
    /// The YAML in the frontmatter is invalid.
    InvalidYaml {
        /// The error message from the YAML parser.
        message: String,
    },
    /// A required field is missing.
    MissingField {
        /// The name of the missing field.
        field: &'static str,
    },
    /// The name field is invalid.
    InvalidName(SkillNameError),
    /// The description field is invalid.
    InvalidDescription(SkillDescriptionError),
    /// The compatibility field is invalid.
    InvalidCompatibility(CompatibilityError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrontmatter => {
                write!(
                    f,
                    "SKILL.md must start with '---' (YAML frontmatter delimiter)"
                )
            }
            Self::UnterminatedFrontmatter => {
                write!(f, "frontmatter must end with '---' on its own line")
            }
            Self::InvalidYaml { message } => {
                write!(f, "invalid YAML in frontmatter: {message}")
            }
            Self::MissingField { field } => {
                write!(f, "missing required field '{field}' in frontmatter")
            }
            Self::InvalidName(e) => write!(f, "invalid name: {e}"),
            Self::InvalidDescription(e) => write!(f, "invalid description: {e}"),
            Self::InvalidCompatibility(e) => write!(f, "invalid compatibility: {e}"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidName(e) => Some(e),
            Self::InvalidDescription(e) => Some(e),
            Self::InvalidCompatibility(e) => Some(e),
            _ => None,
        }
    }
}

impl From<SkillNameError> for ParseError {
    fn from(e: SkillNameError) -> Self {
        Self::InvalidName(e)
    }
}

impl From<SkillDescriptionError> for ParseError {
    fn from(e: SkillDescriptionError) -> Self {
        Self::InvalidDescription(e)
    }
}

impl From<CompatibilityError> for ParseError {
    fn from(e: CompatibilityError) -> Self {
        Self::InvalidCompatibility(e)
    }
}

/// Error returned when loading a skill from a directory fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The directory doesn't exist.
    DirectoryNotFound {
        /// The path that was not found.
        path: String,
    },
    /// The SKILL.md file is missing.
    SkillFileNotFound {
        /// The directory path.
        path: String,
    },
    /// An I/O error occurred.
    IoError {
        /// The path being accessed.
        path: String,
        /// The kind of I/O error.
        kind: std::io::ErrorKind,
        /// The error message.
        message: String,
    },
    /// The SKILL.md file couldn't be parsed.
    ParseError(ParseError),
    /// The skill name doesn't match the directory name.
    NameMismatch {
        /// The directory name.
        directory_name: String,
        /// The skill name from frontmatter.
        skill_name: String,
    },
    /// A referenced file was not found.
    FileNotFound {
        /// The path that was not found.
        path: String,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectoryNotFound { path } => {
                write!(f, "skill directory not found: '{path}'")
            }
            Self::SkillFileNotFound { path } => {
                write!(f, "SKILL.md not found in '{path}'")
            }
            Self::IoError { path, message, .. } => {
                write!(f, "error reading '{path}': {message}")
            }
            Self::ParseError(e) => write!(f, "{e}"),
            Self::NameMismatch {
                directory_name,
                skill_name,
            } => {
                write!(
                    f,
                    "skill name '{skill_name}' must match directory name '{directory_name}' (per specification)"
                )
            }
            Self::FileNotFound { path } => {
                write!(f, "file not found: '{path}'")
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ParseError> for LoadError {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn parse_error_display_is_helpful() {
        let err = ParseError::MissingFrontmatter;
        let msg = err.to_string();
        assert!(msg.contains("---"));
        assert!(msg.contains("SKILL.md"));
    }

    #[test]
    fn parse_error_invalid_yaml_includes_message() {
        let err = ParseError::InvalidYaml {
            message: "expected ':'".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("expected ':'"));
    }

    #[test]
    fn parse_error_missing_field_includes_field_name() {
        let err = ParseError::MissingField { field: "name" };
        let msg = err.to_string();
        assert!(msg.contains("name"));
    }

    #[test]
    fn parse_error_source_returns_inner_error() {
        let inner = SkillNameError::Empty;
        let err = ParseError::InvalidName(inner.clone());
        assert!(err.source().is_some());
    }

    #[test]
    fn load_error_display_is_helpful() {
        let err = LoadError::DirectoryNotFound {
            path: "/path/to/skill".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/path/to/skill"));
    }

    #[test]
    fn load_error_name_mismatch_is_clear() {
        let err = LoadError::NameMismatch {
            directory_name: "my-skill".to_string(),
            skill_name: "other-skill".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("my-skill"));
        assert!(msg.contains("other-skill"));
        assert!(msg.contains("must match"));
    }

    #[test]
    fn load_error_source_returns_parse_error() {
        let inner = ParseError::MissingFrontmatter;
        let err = LoadError::ParseError(inner);
        assert!(err.source().is_some());
    }
}
