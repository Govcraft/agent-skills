//! Exit code types for the CLI.

use crate::error::CliError;

/// Exit codes for the CLI following Unix conventions.
///
/// - 0: Success
/// - 1: General error
/// - 10: Validation error
/// - 11: I/O error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    /// Command completed successfully.
    Success = 0,
    /// General error (unspecified).
    GeneralError = 1,
    /// Validation error (skill failed validation).
    ValidationError = 10,
    /// I/O error (file not found, permission denied, etc.).
    IoError = 11,
}

impl ExitCode {
    /// Returns the numeric exit code.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        Self::from(code.code())
    }
}

impl From<&CliError> for ExitCode {
    fn from(error: &CliError) -> Self {
        match error {
            CliError::LoadError { .. } => Self::ValidationError,
            CliError::SerializationError { .. } | CliError::NoPathsProvided => Self::GeneralError,
            CliError::PathNotFound { .. }
            | CliError::IoError { .. }
            | CliError::StdinError { .. } => Self::IoError,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn exit_code_success_is_zero() {
        assert_eq!(ExitCode::Success.code(), 0);
    }

    #[test]
    fn exit_code_general_error_is_one() {
        assert_eq!(ExitCode::GeneralError.code(), 1);
    }

    #[test]
    fn exit_code_validation_error_is_ten() {
        assert_eq!(ExitCode::ValidationError.code(), 10);
    }

    #[test]
    fn exit_code_io_error_is_eleven() {
        assert_eq!(ExitCode::IoError.code(), 11);
    }

    #[test]
    fn exit_code_from_path_not_found_is_io_error() {
        let err = CliError::PathNotFound {
            path: PathBuf::from("/test"),
        };
        assert_eq!(ExitCode::from(&err), ExitCode::IoError);
    }

    #[test]
    fn exit_code_from_load_error_is_validation_error() {
        let err = CliError::LoadError {
            path: PathBuf::from("/test"),
            message: "test".to_string(),
        };
        assert_eq!(ExitCode::from(&err), ExitCode::ValidationError);
    }

    #[test]
    fn exit_code_from_serialization_error_is_general_error() {
        let err = CliError::SerializationError {
            message: "test".to_string(),
        };
        assert_eq!(ExitCode::from(&err), ExitCode::GeneralError);
    }

    #[test]
    fn exit_code_from_io_error_is_io_error() {
        let err = CliError::IoError {
            path: None,
            kind: io::ErrorKind::NotFound,
            message: "test".to_string(),
        };
        assert_eq!(ExitCode::from(&err), ExitCode::IoError);
    }

    #[test]
    fn exit_code_from_stdin_error_is_io_error() {
        let err = CliError::StdinError {
            message: "test".to_string(),
        };
        assert_eq!(ExitCode::from(&err), ExitCode::IoError);
    }

    #[test]
    fn exit_code_from_no_paths_provided_is_general_error() {
        let err = CliError::NoPathsProvided;
        assert_eq!(ExitCode::from(&err), ExitCode::GeneralError);
    }

    #[test]
    fn exit_code_converts_to_std_process_exit_code() {
        let code: std::process::ExitCode = ExitCode::Success.into();
        // We can't easily inspect the value, but we can verify the conversion compiles
        let _ = code;
    }
}
