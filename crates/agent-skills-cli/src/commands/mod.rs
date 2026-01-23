//! CLI command implementations.

pub mod list;
pub mod read_properties;
pub mod to_prompt;
pub mod validate;

use std::path::{Path, PathBuf};

use crate::error::CliError;

/// Resolves a skill path, handling SKILL.md files.
///
/// If the path points to a SKILL.md file, returns the parent directory.
/// Otherwise, returns the path as-is (if it exists).
///
/// # Errors
///
/// Returns `CliError::PathNotFound` if the path doesn't exist.
pub fn resolve_skill_path(path: &Path) -> Result<PathBuf, CliError> {
    if !path.exists() {
        return Err(CliError::PathNotFound {
            path: path.to_path_buf(),
        });
    }

    // If path is a SKILL.md file, use parent directory
    if path.is_file()
        && path.file_name().is_some_and(|name| name == "SKILL.md")
        && let Some(parent) = path.parent()
    {
        return Ok(parent.to_path_buf());
    }

    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_skill_dir(temp: &TempDir, name: &str) -> PathBuf {
        let skill_dir = temp.path().join(name);
        fs::create_dir(&skill_dir).ok();
        let content = format!(
            r#"---
name: {name}
description: Test skill.
---
# Instructions
"#
        );
        fs::write(skill_dir.join("SKILL.md"), content).ok();
        skill_dir
    }

    #[test]
    fn resolve_skill_path_returns_directory_unchanged() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill");
            let result = resolve_skill_path(&skill_dir);
            assert!(result.is_ok());
            assert_eq!(result.ok(), Some(skill_dir));
        }
    }

    #[test]
    fn resolve_skill_path_returns_parent_for_skill_md() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill");
            let skill_md = skill_dir.join("SKILL.md");
            let result = resolve_skill_path(&skill_md);
            assert!(result.is_ok());
            assert_eq!(result.ok(), Some(skill_dir));
        }
    }

    #[test]
    fn resolve_skill_path_returns_error_for_nonexistent() {
        let result = resolve_skill_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        if let Err(CliError::PathNotFound { path }) = result {
            assert_eq!(path, PathBuf::from("/nonexistent/path"));
        } else {
            panic!("Expected PathNotFound error");
        }
    }
}
