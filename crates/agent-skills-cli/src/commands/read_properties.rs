//! Read-properties command implementation.

use std::collections::HashMap;
use std::io::{self, BufRead};
use std::path::PathBuf;

use agent_skills::SkillDirectory;
use serde::Serialize;

use crate::error::CliError;
use crate::output_format::OutputFormat;
use crate::output_mode::OutputMode;

/// Output format for skill properties.
#[derive(Serialize)]
pub struct SkillProperties {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
}

/// Reads a path from the given source (file path or "-" for stdin).
fn read_path(path_arg: &str) -> Result<PathBuf, CliError> {
    if path_arg == "-" {
        read_path_from_stdin()
    } else {
        Ok(PathBuf::from(path_arg))
    }
}

/// Reads a single path from stdin.
fn read_path_from_stdin() -> Result<PathBuf, CliError> {
    let stdin = io::stdin();
    let line = stdin
        .lock()
        .lines()
        .find_map(|line| match line {
            Ok(l) => {
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(Ok(trimmed.to_string()))
                }
            }
            Err(e) => Some(Err(CliError::StdinError {
                message: e.to_string(),
            })),
        })
        .ok_or(CliError::NoPathsProvided)??;

    Ok(PathBuf::from(line))
}

/// Reads skill properties and outputs in the specified format.
///
/// # Errors
///
/// Returns `CliError` if:
/// - The path doesn't exist
/// - The skill is invalid
/// - Serialization fails
pub fn run(
    path_arg: &str,
    format: OutputFormat,
    compact: bool,
    _output_mode: OutputMode,
) -> Result<(), CliError> {
    let skill_path = read_path(path_arg)?;
    let skill_path = super::resolve_skill_path(&skill_path)?;

    let dir = SkillDirectory::load(&skill_path).map_err(|e| CliError::LoadError {
        path: skill_path.clone(),
        message: e.to_string(),
    })?;

    let skill = dir.skill();
    let frontmatter = skill.frontmatter();

    let properties = SkillProperties {
        name: skill.name().as_str().to_string(),
        description: skill.description().as_str().to_string(),
        license: frontmatter.license().map(String::from),
        compatibility: frontmatter.compatibility().map(|c| c.as_str().to_string()),
        metadata: frontmatter.metadata().map(|m| {
            m.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        }),
    };

    let output = serialize_properties(&properties, format, compact)?;
    println!("{output}");

    Ok(())
}

/// Serializes properties to the specified format.
fn serialize_properties(
    properties: &SkillProperties,
    format: OutputFormat,
    compact: bool,
) -> Result<String, CliError> {
    match format {
        OutputFormat::Json => if compact {
            serde_json::to_string(properties)
        } else {
            serde_json::to_string_pretty(properties)
        }
        .map_err(|e| CliError::SerializationError {
            message: e.to_string(),
        }),
        OutputFormat::Yaml => serde_yml::to_string(properties)
            .map(|s| s.trim_end().to_string())
            .map_err(|e| CliError::SerializationError {
                message: e.to_string(),
            }),
        OutputFormat::Toml => {
            toml::to_string_pretty(properties).map_err(|e| CliError::SerializationError {
                message: e.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_skill_dir(temp: &TempDir, name: &str, content: &str) -> PathBuf {
        let skill_dir = temp.path().join(name);
        fs::create_dir(&skill_dir).ok();
        fs::write(skill_dir.join("SKILL.md"), content).ok();
        skill_dir
    }

    fn minimal_skill_content(name: &str) -> String {
        format!(
            r#"---
name: {name}
description: Test skill.
---
# Instructions
"#
        )
    }

    fn full_skill_content(name: &str) -> String {
        format!(
            r#"---
name: {name}
description: Test skill with all fields.
license: MIT
compatibility: Requires docker
metadata:
  author: test-author
  version: "1.0"
---
# Instructions
"#
        )
    }

    #[test]
    fn run_succeeds_for_minimal_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", &minimal_skill_content("my-skill"));
            let result = run(
                skill_dir.to_str().unwrap_or("."),
                OutputFormat::Json,
                false,
                OutputMode::Quiet,
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_succeeds_for_full_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", &full_skill_content("my-skill"));
            let result = run(
                skill_dir.to_str().unwrap_or("."),
                OutputFormat::Json,
                false,
                OutputMode::Quiet,
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_fails_for_nonexistent_path() {
        let result = run(
            "/nonexistent/path",
            OutputFormat::Json,
            false,
            OutputMode::Quiet,
        );
        assert!(result.is_err());
    }

    #[test]
    fn skill_properties_serializes_minimal() {
        let props = SkillProperties {
            name: "test".to_string(),
            description: "Test description.".to_string(),
            license: None,
            compatibility: None,
            metadata: None,
        };

        let json = serde_json::to_string(&props);
        assert!(json.is_ok());
        let json = json.ok();
        if let Some(json) = json {
            assert!(json.contains("\"name\":\"test\""));
            assert!(json.contains("\"description\":\"Test description.\""));
            // Optional fields should not appear
            assert!(!json.contains("license"));
            assert!(!json.contains("compatibility"));
            assert!(!json.contains("metadata"));
        }
    }

    #[test]
    fn skill_properties_serializes_full() {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "test".to_string());

        let props = SkillProperties {
            name: "test".to_string(),
            description: "Test description.".to_string(),
            license: Some("MIT".to_string()),
            compatibility: Some("Requires docker".to_string()),
            metadata: Some(metadata),
        };

        let json = serde_json::to_string(&props);
        assert!(json.is_ok());
        let json = json.ok();
        if let Some(json) = json {
            assert!(json.contains("\"license\":\"MIT\""));
            assert!(json.contains("\"compatibility\":\"Requires docker\""));
            assert!(json.contains("\"author\":\"test\""));
        }
    }

    #[test]
    fn serialize_properties_json_pretty() {
        let props = SkillProperties {
            name: "test".to_string(),
            description: "Test.".to_string(),
            license: None,
            compatibility: None,
            metadata: None,
        };

        let result = serialize_properties(&props, OutputFormat::Json, false);
        assert!(result.is_ok());
        let json = result.ok();
        if let Some(json) = json {
            // Pretty format has newlines
            assert!(json.contains('\n'));
        }
    }

    #[test]
    fn serialize_properties_json_compact() {
        let props = SkillProperties {
            name: "test".to_string(),
            description: "Test.".to_string(),
            license: None,
            compatibility: None,
            metadata: None,
        };

        let result = serialize_properties(&props, OutputFormat::Json, true);
        assert!(result.is_ok());
        let json = result.ok();
        if let Some(json) = json {
            // Compact format has no newlines
            assert!(!json.contains('\n'));
        }
    }

    #[test]
    fn serialize_properties_yaml() {
        let props = SkillProperties {
            name: "test".to_string(),
            description: "Test.".to_string(),
            license: Some("MIT".to_string()),
            compatibility: None,
            metadata: None,
        };

        let result = serialize_properties(&props, OutputFormat::Yaml, false);
        assert!(result.is_ok());
        let yaml = result.ok();
        if let Some(yaml) = yaml {
            assert!(yaml.contains("name: test"));
            assert!(yaml.contains("license: MIT"));
        }
    }

    #[test]
    fn serialize_properties_toml() {
        let props = SkillProperties {
            name: "test".to_string(),
            description: "Test.".to_string(),
            license: Some("MIT".to_string()),
            compatibility: None,
            metadata: None,
        };

        let result = serialize_properties(&props, OutputFormat::Toml, false);
        assert!(result.is_ok());
        let toml_str = result.ok();
        if let Some(toml_str) = toml_str {
            assert!(toml_str.contains("name = \"test\""));
            assert!(toml_str.contains("license = \"MIT\""));
        }
    }

    #[test]
    fn read_path_returns_path_for_regular_arg() {
        let path = read_path("/some/path");
        assert!(path.is_ok());
        assert_eq!(path.ok(), Some(PathBuf::from("/some/path")));
    }
}
