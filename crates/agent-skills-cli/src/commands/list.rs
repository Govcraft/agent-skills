//! List command implementation.

use std::path::Path;

use agent_skills::SkillDirectory;
use serde::Serialize;

use crate::error::CliError;
use crate::output_mode::OutputMode;
use crate::ListFormat;

/// Information about a discovered skill.
#[derive(Serialize)]
pub struct SkillInfo {
    name: String,
    description: String,
    path: String,
}

/// Lists skills in a directory.
///
/// # Errors
///
/// Returns `CliError` if:
/// - The directory doesn't exist
/// - I/O errors occur during traversal
pub fn run(
    directory: &Path,
    recursive: bool,
    format: ListFormat,
    output_mode: OutputMode,
) -> Result<(), CliError> {
    if !directory.exists() {
        return Err(CliError::PathNotFound {
            path: directory.to_path_buf(),
        });
    }

    let skills = discover_skills(directory, recursive)?;

    if skills.is_empty() {
        if output_mode.show_info() {
            eprintln!("No skills found in '{}'", directory.display());
        }
        return Ok(());
    }

    match format {
        ListFormat::Text => {
            for info in &skills {
                println!("{}\t{}\t{}", info.name, info.path, info.description);
            }
        }
        ListFormat::Json => {
            let json = serde_json::to_string_pretty(&skills).map_err(|e| {
                CliError::SerializationError {
                    message: e.to_string(),
                }
            })?;
            println!("{json}");
        }
    }

    if output_mode.show_info() {
        eprintln!("Found {} skill(s)", skills.len());
    }

    Ok(())
}

/// Discovers skills in a directory.
fn discover_skills(directory: &Path, recursive: bool) -> Result<Vec<SkillInfo>, CliError> {
    let mut skills = Vec::new();

    discover_skills_in_dir(directory, recursive, &mut skills)?;

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Recursively discovers skills in a directory.
fn discover_skills_in_dir(
    directory: &Path,
    recursive: bool,
    skills: &mut Vec<SkillInfo>,
) -> Result<(), CliError> {
    let entries = std::fs::read_dir(directory).map_err(|e| CliError::IoError {
        path: Some(directory.to_path_buf()),
        kind: e.kind(),
        message: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| CliError::IoError {
            path: Some(directory.to_path_buf()),
            kind: e.kind(),
            message: e.to_string(),
        })?;

        let path = entry.path();

        if path.is_dir() {
            // Check if this directory is a skill (has SKILL.md)
            let skill_md = path.join("SKILL.md");
            if skill_md.exists()
                && let Ok(dir) = SkillDirectory::load(&path)
            {
                let skill = dir.skill();
                skills.push(SkillInfo {
                    name: skill.name().as_str().to_string(),
                    description: skill.description().as_str().to_string(),
                    path: path.display().to_string(),
                });
            }

            // Recurse into subdirectories if requested
            if recursive {
                discover_skills_in_dir(&path, recursive, skills)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_skill_dir(dir: &Path, name: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).ok();
        let content = format!(
            r#"---
name: {name}
description: Test skill for {name}.
---
# Instructions
"#
        );
        fs::write(skill_dir.join("SKILL.md"), content).ok();
    }

    #[test]
    fn discover_skills_finds_skills_in_directory() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            create_skill_dir(temp.path(), "my-skill");

            let skills = discover_skills(temp.path(), false);
            assert!(skills.is_ok());
            let skills = skills.ok();
            if let Some(skills) = skills {
                assert_eq!(skills.len(), 1);
                assert_eq!(skills[0].name, "my-skill");
            }
        }
    }

    #[test]
    fn discover_skills_returns_empty_for_empty_directory() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            let skills = discover_skills(temp.path(), false);
            assert!(skills.is_ok());
            let skills = skills.ok();
            if let Some(skills) = skills {
                assert!(skills.is_empty());
            }
        }
    }

    #[test]
    fn discover_skills_recursive_finds_nested_skills() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            // Create nested skill directories
            let nested = temp.path().join("plugins");
            fs::create_dir(&nested).ok();
            create_skill_dir(&nested, "nested-skill");

            let skills_non_recursive = discover_skills(temp.path(), false);
            let skills_recursive = discover_skills(temp.path(), true);

            assert!(skills_non_recursive.is_ok());
            assert!(skills_recursive.is_ok());

            if let (Some(non_recursive), Some(recursive)) =
                (skills_non_recursive.ok(), skills_recursive.ok())
            {
                assert!(non_recursive.is_empty());
                assert_eq!(recursive.len(), 1);
                assert_eq!(recursive[0].name, "nested-skill");
            }
        }
    }

    #[test]
    fn discover_skills_sorts_alphabetically() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            create_skill_dir(temp.path(), "z-skill");
            create_skill_dir(temp.path(), "a-skill");
            create_skill_dir(temp.path(), "m-skill");

            let skills = discover_skills(temp.path(), false);
            assert!(skills.is_ok());
            let skills = skills.ok();
            if let Some(skills) = skills {
                assert_eq!(skills.len(), 3);
                assert_eq!(skills[0].name, "a-skill");
                assert_eq!(skills[1].name, "m-skill");
                assert_eq!(skills[2].name, "z-skill");
            }
        }
    }

    #[test]
    fn run_returns_error_for_nonexistent_directory() {
        let result = run(
            Path::new("/nonexistent/path"),
            false,
            ListFormat::Text,
            OutputMode::Normal,
        );
        assert!(result.is_err());
        if let Err(CliError::PathNotFound { .. }) = result {
            // Expected
        } else {
            panic!("Expected PathNotFound error");
        }
    }

    #[test]
    fn run_succeeds_for_empty_directory() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            let result = run(temp.path(), false, ListFormat::Text, OutputMode::Quiet);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn skill_info_serializes_to_json() {
        let info = SkillInfo {
            name: "test-skill".to_string(),
            description: "A test skill.".to_string(),
            path: "/path/to/skill".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("\"name\":\"test-skill\""));
        assert!(json.contains("\"description\":\"A test skill.\""));
        assert!(json.contains("\"path\":\"/path/to/skill\""));
    }
}
