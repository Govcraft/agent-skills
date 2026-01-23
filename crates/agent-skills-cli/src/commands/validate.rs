//! Validate command implementation.

use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use agent_skills::SkillDirectory;

use crate::color::{self, ColorConfig};
use crate::error::CliError;
use crate::output_mode::OutputMode;

/// Reads paths from the given source (file path or "-" for stdin).
fn read_paths(path_arg: &str) -> Result<Vec<PathBuf>, CliError> {
    if path_arg == "-" {
        read_paths_from_stdin()
    } else {
        Ok(vec![PathBuf::from(path_arg)])
    }
}

/// Reads paths from stdin, one per line.
fn read_paths_from_stdin() -> Result<Vec<PathBuf>, CliError> {
    let stdin = io::stdin();
    let paths: Result<Vec<_>, _> = stdin
        .lock()
        .lines()
        .filter_map(|line| match line {
            Ok(l) => {
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(Ok(PathBuf::from(trimmed)))
                }
            }
            Err(e) => Some(Err(CliError::StdinError {
                message: e.to_string(),
            })),
        })
        .collect();

    let paths = paths?;

    if paths.is_empty() {
        return Err(CliError::NoPathsProvided);
    }

    Ok(paths)
}

/// Validates skill directories.
///
/// Outputs "Valid skill: <path>" to stderr on success (respects output mode).
///
/// # Errors
///
/// Returns `CliError` if:
/// - The path doesn't exist
/// - The skill is invalid
/// - Reading from stdin fails
pub fn run(
    path_arg: &str,
    output_mode: OutputMode,
    color_config: ColorConfig,
) -> Result<(), CliError> {
    let paths = read_paths(path_arg)?;

    for (i, path) in paths.iter().enumerate() {
        if output_mode.show_progress() && paths.len() > 1 {
            let colored_name = color::path(&path.display().to_string(), color_config);
            let count_info = color::summary(&format!("[{}/{}]", i + 1, paths.len()), color_config);
            eprintln!("{count_info} Validating: {colored_name}");
        }

        validate_single(path)?;

        if output_mode.show_info() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("skill");
            let symbol = color::success_symbol(color_config);
            let colored_name = color::skill_name(name, color_config);
            let colored_path = color::path(&format!("({})", path.display()), color_config);
            eprintln!("{symbol} {colored_name} {colored_path}");
        }
    }

    Ok(())
}

/// Validates a single skill path.
fn validate_single(skill_path: &Path) -> Result<(), CliError> {
    let skill_path = super::resolve_skill_path(skill_path)?;

    SkillDirectory::load(&skill_path).map_err(|e| CliError::LoadError {
        path: skill_path.clone(),
        message: e.to_string(),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorConfig;
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

    #[test]
    fn run_succeeds_for_valid_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", &minimal_skill_content("my-skill"));
            let result = run(
                skill_dir.to_str().unwrap_or("."),
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_fails_for_invalid_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", "invalid content");
            let result = run(
                skill_dir.to_str().unwrap_or("."),
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_err());
        }
    }

    #[test]
    fn run_fails_for_nonexistent_path() {
        let result = run(
            "/nonexistent/path",
            OutputMode::Quiet,
            ColorConfig::default(),
        );
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
            let result = run(
                skill_md.to_str().unwrap_or("."),
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn read_paths_returns_single_path_for_regular_arg() {
        let paths = read_paths("/some/path");
        assert!(paths.is_ok());
        let paths = paths.ok();
        if let Some(paths) = paths {
            assert_eq!(paths.len(), 1);
            assert_eq!(paths[0], PathBuf::from("/some/path"));
        }
    }

    #[test]
    fn validate_single_succeeds_for_valid_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", &minimal_skill_content("my-skill"));
            let result = validate_single(&skill_dir);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn validate_single_fails_for_invalid_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", "invalid content");
            let result = validate_single(&skill_dir);
            assert!(result.is_err());
            if let Err(CliError::LoadError { .. }) = result {
                // Expected
            } else {
                panic!("Expected LoadError");
            }
        }
    }
}
