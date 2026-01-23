//! Output format types for structured data.

use std::fmt;
use std::str::FromStr;

/// Output format for structured data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// JSON format (default).
    #[default]
    Json,
    /// YAML format.
    Yaml,
    /// TOML format.
    Toml,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Toml => write!(f, "toml"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = OutputFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            "toml" => Ok(Self::Toml),
            _ => Err(OutputFormatError {
                value: s.to_string(),
            }),
        }
    }
}

/// Error for invalid output format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFormatError {
    /// The invalid value that was provided.
    pub value: String,
}

impl fmt::Display for OutputFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid output format '{}'; valid formats are: json, yaml, toml",
            self.value
        )
    }
}

impl std::error::Error for OutputFormatError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_default_is_json() {
        assert_eq!(OutputFormat::default(), OutputFormat::Json);
    }

    #[test]
    fn output_format_display_json() {
        assert_eq!(OutputFormat::Json.to_string(), "json");
    }

    #[test]
    fn output_format_display_yaml() {
        assert_eq!(OutputFormat::Yaml.to_string(), "yaml");
    }

    #[test]
    fn output_format_display_toml() {
        assert_eq!(OutputFormat::Toml.to_string(), "toml");
    }

    #[test]
    fn output_format_parses_json() {
        assert_eq!(
            "json".parse::<OutputFormat>().ok(),
            Some(OutputFormat::Json)
        );
    }

    #[test]
    fn output_format_parses_yaml() {
        assert_eq!(
            "yaml".parse::<OutputFormat>().ok(),
            Some(OutputFormat::Yaml)
        );
    }

    #[test]
    fn output_format_parses_yml() {
        assert_eq!("yml".parse::<OutputFormat>().ok(), Some(OutputFormat::Yaml));
    }

    #[test]
    fn output_format_parses_toml() {
        assert_eq!(
            "toml".parse::<OutputFormat>().ok(),
            Some(OutputFormat::Toml)
        );
    }

    #[test]
    fn output_format_parses_case_insensitive() {
        assert_eq!(
            "JSON".parse::<OutputFormat>().ok(),
            Some(OutputFormat::Json)
        );
        assert_eq!(
            "YAML".parse::<OutputFormat>().ok(),
            Some(OutputFormat::Yaml)
        );
        assert_eq!(
            "TOML".parse::<OutputFormat>().ok(),
            Some(OutputFormat::Toml)
        );
    }

    #[test]
    fn output_format_rejects_invalid() {
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn output_format_error_display() {
        let err = OutputFormatError {
            value: "invalid".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("invalid"));
        assert!(msg.contains("json"));
        assert!(msg.contains("yaml"));
        assert!(msg.contains("toml"));
    }
}
