//! CLI error types.

use std::fmt;
use std::io;
use std::path::PathBuf;

use serde::Serialize;

/// Error codes for structured error output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Path does not exist.
    PathNotFound,
    /// Skill failed to load/validate.
    LoadError,
    /// Serialization failed.
    SerializationError,
    /// I/O operation failed.
    IoError,
    /// Failed to read from stdin.
    StdinError,
    /// No paths provided when required.
    NoPathsProvided,
}

/// Errors that can occur when running CLI commands.
#[derive(Debug)]
pub enum CliError {
    /// A skill path was not found.
    PathNotFound {
        /// The path that was not found.
        path: PathBuf,
    },
    /// A skill directory could not be loaded.
    LoadError {
        /// The path to the skill directory.
        path: PathBuf,
        /// The error message from the loader.
        message: String,
    },
    /// Failed to serialize output.
    SerializationError {
        /// The error message from serialization.
        message: String,
    },
    /// An I/O error occurred.
    IoError {
        /// The path being accessed, if known.
        path: Option<PathBuf>,
        /// The kind of I/O error.
        kind: io::ErrorKind,
        /// The error message.
        message: String,
    },
    /// Failed to read from stdin.
    StdinError {
        /// The error message.
        message: String,
    },
    /// No paths provided when required.
    NoPathsProvided,
}

impl CliError {
    /// Returns the error code for this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::PathNotFound { .. } => ErrorCode::PathNotFound,
            Self::LoadError { .. } => ErrorCode::LoadError,
            Self::SerializationError { .. } => ErrorCode::SerializationError,
            Self::IoError { .. } => ErrorCode::IoError,
            Self::StdinError { .. } => ErrorCode::StdinError,
            Self::NoPathsProvided => ErrorCode::NoPathsProvided,
        }
    }

    /// Returns a suggestion for how to fix this error.
    #[must_use]
    pub const fn suggestion(&self) -> &'static str {
        match self {
            Self::PathNotFound { .. } => {
                "Ensure the path points to a valid skill directory containing SKILL.md."
            }
            Self::LoadError { .. } => {
                "Check that SKILL.md contains valid YAML frontmatter with 'name' and 'description' fields."
            }
            Self::SerializationError { .. } => {
                "This is an internal error. Please report it if it persists."
            }
            Self::IoError { kind, .. } => match kind {
                io::ErrorKind::NotFound => "Verify the path exists and is accessible.",
                io::ErrorKind::PermissionDenied => {
                    "Check file permissions or run with appropriate privileges."
                }
                _ => "Check that the file is accessible and try again.",
            },
            Self::StdinError { .. } => "Ensure stdin contains valid paths, one per line.",
            Self::NoPathsProvided => {
                "Provide at least one path, or use '-' to read paths from stdin."
            }
        }
    }

    /// Converts error to JSON for structured output.
    #[must_use]
    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct JsonError<'a> {
            error: ErrorDetails<'a>,
        }

        #[derive(Serialize)]
        struct ErrorDetails<'a> {
            code: ErrorCode,
            message: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            path: Option<&'a str>,
            suggestion: &'static str,
        }

        let path_string = match self {
            Self::PathNotFound { path } | Self::LoadError { path, .. } => {
                Some(path.display().to_string())
            }
            Self::IoError { path: Some(p), .. } => Some(p.display().to_string()),
            _ => None,
        };

        let details = ErrorDetails {
            code: self.code(),
            message: self.to_string(),
            path: path_string.as_deref(),
            suggestion: self.suggestion(),
        };

        // Safe: our types are always serializable
        serde_json::to_string(&JsonError { error: details }).unwrap_or_else(|_| {
            format!(r#"{{"error":{{"code":"SERIALIZATION_ERROR","message":"{self}"}}}}"#)
        })
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathNotFound { path } => {
                write!(f, "path not found: '{}'", path.display())
            }
            Self::LoadError { path, message } => {
                write!(f, "failed to load skill at '{}': {message}", path.display())
            }
            Self::SerializationError { message } => {
                write!(f, "serialization error: {message}")
            }
            Self::IoError {
                path,
                kind,
                message,
            } => {
                if let Some(p) = path {
                    write!(f, "I/O error ({kind:?}) on '{}': {message}", p.display())
                } else {
                    write!(f, "I/O error ({kind:?}): {message}")
                }
            }
            Self::StdinError { message } => {
                write!(f, "failed to read from stdin: {message}")
            }
            Self::NoPathsProvided => {
                write!(f, "no paths provided")
            }
        }
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        Self::IoError {
            path: None,
            kind: error.kind(),
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_not_found_display_includes_path() {
        let error = CliError::PathNotFound {
            path: PathBuf::from("/some/path"),
        };
        let msg = error.to_string();
        assert!(msg.contains("/some/path"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn load_error_display_includes_path_and_message() {
        let error = CliError::LoadError {
            path: PathBuf::from("/skill/path"),
            message: "SKILL.md not found".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("/skill/path"));
        assert!(msg.contains("SKILL.md not found"));
    }

    #[test]
    fn serialization_error_display_includes_message() {
        let error = CliError::SerializationError {
            message: "invalid JSON".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("invalid JSON"));
    }

    #[test]
    fn io_error_with_path_display_includes_path() {
        let error = CliError::IoError {
            path: Some(PathBuf::from("/file/path")),
            kind: io::ErrorKind::PermissionDenied,
            message: "permission denied".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("/file/path"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn io_error_without_path_display_works() {
        let error = CliError::IoError {
            path: None,
            kind: io::ErrorKind::Other,
            message: "some error".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("some error"));
    }

    #[test]
    fn stdin_error_display_includes_message() {
        let error = CliError::StdinError {
            message: "read error".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("failed to read from stdin"));
        assert!(msg.contains("read error"));
    }

    #[test]
    fn no_paths_provided_display() {
        let error = CliError::NoPathsProvided;
        let msg = error.to_string();
        assert!(msg.contains("no paths provided"));
    }

    #[test]
    fn from_io_error_preserves_kind_and_message() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let cli_err = CliError::from(io_err);

        if let CliError::IoError { kind, message, .. } = cli_err {
            assert_eq!(kind, io::ErrorKind::NotFound);
            assert!(message.contains("file not found"));
        } else {
            panic!("Expected IoError variant");
        }
    }

    #[test]
    fn error_code_returns_correct_code() {
        assert_eq!(
            CliError::PathNotFound {
                path: PathBuf::new()
            }
            .code(),
            ErrorCode::PathNotFound
        );
        assert_eq!(
            CliError::LoadError {
                path: PathBuf::new(),
                message: String::new()
            }
            .code(),
            ErrorCode::LoadError
        );
        assert_eq!(
            CliError::SerializationError {
                message: String::new()
            }
            .code(),
            ErrorCode::SerializationError
        );
        assert_eq!(
            CliError::IoError {
                path: None,
                kind: io::ErrorKind::Other,
                message: String::new()
            }
            .code(),
            ErrorCode::IoError
        );
        assert_eq!(
            CliError::StdinError {
                message: String::new()
            }
            .code(),
            ErrorCode::StdinError
        );
        assert_eq!(CliError::NoPathsProvided.code(), ErrorCode::NoPathsProvided);
    }

    #[test]
    fn path_not_found_suggestion_mentions_skill_md() {
        let err = CliError::PathNotFound {
            path: PathBuf::from("/test"),
        };
        assert!(err.suggestion().contains("SKILL.md"));
    }

    #[test]
    fn load_error_suggestion_mentions_yaml_frontmatter() {
        let err = CliError::LoadError {
            path: PathBuf::from("/test"),
            message: String::new(),
        };
        assert!(err.suggestion().contains("YAML frontmatter"));
    }

    #[test]
    fn io_error_not_found_suggestion() {
        let err = CliError::IoError {
            path: None,
            kind: io::ErrorKind::NotFound,
            message: String::new(),
        };
        assert!(err.suggestion().contains("exists"));
    }

    #[test]
    fn io_error_permission_denied_suggestion() {
        let err = CliError::IoError {
            path: None,
            kind: io::ErrorKind::PermissionDenied,
            message: String::new(),
        };
        assert!(err.suggestion().contains("permission"));
    }

    #[test]
    fn no_paths_provided_suggestion_mentions_stdin() {
        let err = CliError::NoPathsProvided;
        assert!(err.suggestion().contains("'-'"));
        assert!(err.suggestion().contains("stdin"));
    }

    #[test]
    fn cli_error_to_json_is_valid_json() {
        let err = CliError::PathNotFound {
            path: PathBuf::from("/test"),
        };
        let json = err.to_json();
        assert!(json.contains("PATH_NOT_FOUND"));
        assert!(json.contains("error"));
        // Verify it parses
        let _: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    }

    #[test]
    fn cli_error_to_json_includes_path() {
        let err = CliError::PathNotFound {
            path: PathBuf::from("/my/path"),
        };
        let json = err.to_json();
        assert!(json.contains("/my/path"));
    }

    #[test]
    fn cli_error_to_json_includes_suggestion() {
        let err = CliError::PathNotFound {
            path: PathBuf::from("/test"),
        };
        let json = err.to_json();
        assert!(json.contains("suggestion"));
        assert!(json.contains("SKILL.md"));
    }

    #[test]
    fn error_code_serializes_screaming_snake_case() {
        let json = serde_json::to_string(&ErrorCode::PathNotFound).expect("serialize");
        assert_eq!(json, "\"PATH_NOT_FOUND\"");

        let json = serde_json::to_string(&ErrorCode::LoadError).expect("serialize");
        assert_eq!(json, "\"LOAD_ERROR\"");

        let json = serde_json::to_string(&ErrorCode::NoPathsProvided).expect("serialize");
        assert_eq!(json, "\"NO_PATHS_PROVIDED\"");
    }
}
