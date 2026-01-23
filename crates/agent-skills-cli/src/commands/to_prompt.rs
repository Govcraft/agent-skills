//! To-prompt command implementation.

use std::io::{self, BufRead};
use std::path::PathBuf;

use agent_skills::SkillDirectory;

use crate::error::CliError;
use crate::output_mode::OutputMode;
use crate::xml::escape_xml;

/// Reads paths from arguments or stdin.
fn read_paths(path_args: &[String]) -> Result<Vec<PathBuf>, CliError> {
    if path_args.len() == 1 && path_args[0] == "-" {
        read_paths_from_stdin()
    } else {
        Ok(path_args.iter().map(PathBuf::from).collect())
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

/// Generates `available_skills` XML block for agent prompts.
///
/// # Errors
///
/// Returns `CliError` if:
/// - Any path doesn't exist
/// - Any skill is invalid
/// - Path canonicalization fails
/// - Reading from stdin fails
pub fn run(path_args: &[String], output_mode: OutputMode) -> Result<(), CliError> {
    let paths = read_paths(path_args)?;
    let mut skills_xml = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        if output_mode.show_progress() && paths.len() > 1 {
            eprintln!(
                "[{}/{}] Processing: {}",
                i + 1,
                paths.len(),
                path.display()
            );
        }

        let skill_path = super::resolve_skill_path(path)?;
        let absolute_path = std::fs::canonicalize(&skill_path).map_err(|e| CliError::IoError {
            path: Some(skill_path.clone()),
            kind: e.kind(),
            message: e.to_string(),
        })?;

        let dir = SkillDirectory::load(&skill_path).map_err(|e| CliError::LoadError {
            path: skill_path.clone(),
            message: e.to_string(),
        })?;

        let skill = dir.skill();
        let skill_md_path = absolute_path.join("SKILL.md");

        skills_xml.push(format_skill_xml(
            skill.name().as_str(),
            skill.description().as_str(),
            &skill_md_path.display().to_string(),
        ));
    }

    println!("<available_skills>");
    for xml in skills_xml {
        print!("{xml}");
    }
    println!("</available_skills>");

    Ok(())
}

/// Formats a single skill as XML.
fn format_skill_xml(name: &str, description: &str, location: &str) -> String {
    format!(
        "<skill>\n<name>{}</name>\n<description>{}</description>\n<location>{}</location>\n</skill>\n",
        escape_xml(name),
        escape_xml(description),
        escape_xml(location),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_skill_dir(temp: &TempDir, name: &str, description: &str) -> PathBuf {
        let skill_dir = temp.path().join(name);
        fs::create_dir(&skill_dir).ok();
        let content = format!(
            r#"---
name: {name}
description: {description}
---
# Instructions
"#
        );
        fs::write(skill_dir.join("SKILL.md"), content).ok();
        skill_dir
    }

    #[test]
    fn format_skill_xml_basic() {
        let xml = format_skill_xml("my-skill", "Does something useful.", "/path/to/skill/SKILL.md");
        assert!(xml.contains("<name>my-skill</name>"));
        assert!(xml.contains("<description>Does something useful.</description>"));
        assert!(xml.contains("<location>/path/to/skill/SKILL.md</location>"));
    }

    #[test]
    fn format_skill_xml_escapes_special_chars() {
        let xml = format_skill_xml("my-skill", "Does <things> & stuff", "/path/to/skill/SKILL.md");
        assert!(xml.contains("&lt;things&gt;"));
        assert!(xml.contains("&amp;"));
    }

    #[test]
    fn run_succeeds_for_single_skill() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill_dir = create_skill_dir(temp, "my-skill", "Test skill.");
            let result = run(
                &[skill_dir.to_str().unwrap_or(".").to_string()],
                OutputMode::Quiet,
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_succeeds_for_multiple_skills() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let skill1 = create_skill_dir(temp, "skill-one", "First skill.");
            let skill2 = create_skill_dir(temp, "skill-two", "Second skill.");
            let result = run(
                &[
                    skill1.to_str().unwrap_or(".").to_string(),
                    skill2.to_str().unwrap_or(".").to_string(),
                ],
                OutputMode::Quiet,
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_fails_for_nonexistent_path() {
        let result = run(&["/nonexistent/path".to_string()], OutputMode::Quiet);
        assert!(result.is_err());
    }

    #[test]
    fn run_fails_if_any_skill_invalid() {
        let temp = TempDir::new().ok();
        let temp = temp.as_ref();
        if let Some(temp) = temp {
            let valid_skill = create_skill_dir(temp, "valid-skill", "Valid skill.");
            let invalid_skill = temp.path().join("invalid-skill");
            fs::create_dir(&invalid_skill).ok();
            fs::write(invalid_skill.join("SKILL.md"), "invalid").ok();

            let result = run(
                &[
                    valid_skill.to_str().unwrap_or(".").to_string(),
                    invalid_skill.to_str().unwrap_or(".").to_string(),
                ],
                OutputMode::Quiet,
            );
            assert!(result.is_err());
        }
    }

    #[test]
    fn read_paths_returns_paths_for_regular_args() {
        let paths = read_paths(&["/path/one".to_string(), "/path/two".to_string()]);
        assert!(paths.is_ok());
        let paths = paths.ok();
        if let Some(paths) = paths {
            assert_eq!(paths.len(), 2);
            assert_eq!(paths[0], PathBuf::from("/path/one"));
            assert_eq!(paths[1], PathBuf::from("/path/two"));
        }
    }
}
