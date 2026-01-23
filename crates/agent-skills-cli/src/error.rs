//! CLI error types.

use std::fmt;
use std::io;
use std::path::PathBuf;

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
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathNotFound { path } => {
                write!(f, "path not found: '{}'", path.display())
            }
            Self::LoadError { path, message } => {
                write!(
                    f,
                    "failed to load skill at '{}': {message}",
                    path.display()
                )
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
}
