//! Validate command implementation.

use std::path::Path;

use agent_skills::SkillDirectory;

use crate::error::CliError;

/// Validates a skill directory.
///
/// Outputs "Valid skill: <path>" to stdout on success.
///
/// # Errors
///
/// Returns `CliError` if:
/// - The path doesn't exist
/// - The skill is invalid
pub fn run(skill_path: &Path) -> Result<(), CliError> {
    let skill_path = super::resolve_skill_path(skill_path)?;

    SkillDirectory::load(&skill_path).map_err(|e| CliError::LoadError {
        path: skill_path.clone(),
        message: e.to_string(),
    })?;

    println!("Valid skill: {}", skill_path.display());
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

    #[test]
    fn run_succeeds_for_valid_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", &minimal_skill_content("my-skill"));
            let result = run(&skill_dir);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_fails_for_invalid_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", "invalid content");
            let result = run(&skill_dir);
            assert!(result.is_err());
        }
    }

    #[test]
    fn run_fails_for_nonexistent_path() {
        let result = run(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        if let Err(CliError::PathNotFound { .. }) = result {
            // Expected
        } else {
            panic!("Expected PathNotFound error");
        }
    }

    #[test]
    fn run_succeeds_with_skill_md_path() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", &minimal_skill_content("my-skill"));
            let skill_md = skill_dir.join("SKILL.md");
            let result = run(&skill_md);
            assert!(result.is_ok());
        }
    }
}
