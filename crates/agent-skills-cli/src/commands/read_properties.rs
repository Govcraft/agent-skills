//! Read-properties command implementation.

use std::collections::HashMap;
use std::path::Path;

use agent_skills::SkillDirectory;
use serde::Serialize;

use crate::error::CliError;

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

/// Reads skill properties and outputs as JSON.
///
/// # Errors
///
/// Returns `CliError` if:
/// - The path doesn't exist
/// - The skill is invalid
/// - JSON serialization fails
pub fn run(skill_path: &Path) -> Result<(), CliError> {
    let skill_path = super::resolve_skill_path(skill_path)?;

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

    let json = serde_json::to_string_pretty(&properties).map_err(|e| {
        CliError::SerializationError {
            message: e.to_string(),
        }
    })?;

    println!("{json}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_skill_dir(temp: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
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
            let result = run(&skill_dir);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_succeeds_for_full_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", &full_skill_content("my-skill"));
            let result = run(&skill_dir);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_fails_for_nonexistent_path() {
        let result = run(Path::new("/nonexistent/path"));
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
}
