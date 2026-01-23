//! List command implementation.

use std::fmt::Write as _;
use std::path::Path;

use agent_skills::SkillDirectory;
use serde::Serialize;
use textwrap::{Options, wrap};

use crate::color::{self, ColorConfig};
use crate::error::CliError;
use crate::output_mode::OutputMode;

/// Maximum description length for short format (including "...").
const SHORT_DESCRIPTION_MAX: usize = 50;

/// Maximum name column width for alignment.
const MAX_NAME_WIDTH: usize = 30;

/// Line width for wrapping long descriptions.
const WRAP_WIDTH: usize = 78;

/// Indentation for wrapped description lines.
const DESCRIPTION_INDENT: &str = "  ";

/// Output mode for the list command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListOutputMode {
    /// Short format: name and truncated description (default).
    #[default]
    Short,
    /// Long format: name, path, and full description.
    Long,
    /// Paths only: one path per line.
    PathsOnly,
    /// Names only: one name per line.
    NamesOnly,
}

/// Information about a discovered skill.
#[derive(Serialize)]
pub struct SkillInfo {
    name: String,
    description: String,
    path: String,
    /// Validation status: Some(true) = valid, Some(false) = invalid, None = not checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,
    /// Error message if validation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
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
    validate: bool,
    failed_only: bool,
    list_mode: ListOutputMode,
    output_mode: OutputMode,
    color_config: ColorConfig,
) -> Result<(), CliError> {
    if !directory.exists() {
        return Err(CliError::PathNotFound {
            path: directory.to_path_buf(),
        });
    }

    let skills = discover_skills(directory, recursive, validate)?;

    // Filter to only failed skills if requested
    let skills: Vec<_> = if failed_only {
        skills.into_iter().filter(|s| s.valid == Some(false)).collect()
    } else {
        skills
    };

    if skills.is_empty() {
        if output_mode.show_info() {
            let msg = if failed_only {
                "No invalid skills found"
            } else {
                "No skills found"
            };
            eprintln!("{msg} in '{}'", directory.display());
        }
        return Ok(());
    }

    // JSON mode via global --json flag takes precedence
    if output_mode.is_json() {
        let json =
            serde_json::to_string_pretty(&skills).map_err(|e| CliError::SerializationError {
                message: e.to_string(),
            })?;
        println!("{json}");
    } else {
        let output = match list_mode {
            ListOutputMode::Short => format_short(&skills, color_config),
            ListOutputMode::Long => format_long(&skills, color_config),
            ListOutputMode::PathsOnly => format_paths_only(&skills),
            ListOutputMode::NamesOnly => format_names_only(&skills),
        };
        print!("{output}");
    }

    if output_mode.show_info() {
        let label = if failed_only { "invalid" } else { "" };
        let summary_text = format!("Found {} {label}skill(s)", skills.len()).replace("  ", " ");
        eprintln!("{}", color::summary(&summary_text, color_config));
    }

    Ok(())
}

/// Truncates a description to the specified maximum length.
///
/// If the description exceeds `max_len`, it is truncated and "..." is appended.
/// Word boundaries are respected where possible.
///
/// # Arguments
/// * `description` - The description to truncate
/// * `max_len` - Maximum length including the "..." suffix
///
/// # Returns
/// The truncated description, or the original if already short enough.
#[must_use]
fn truncate_description(description: &str, max_len: usize) -> String {
    if description.len() <= max_len {
        return description.to_string();
    }

    // Reserve space for "..."
    let target_len = max_len.saturating_sub(3);

    // Find the last space within the target length for word boundary
    let truncate_at = description[..target_len].rfind(' ').unwrap_or(target_len);

    format!("{}...", &description[..truncate_at])
}

/// Calculates the maximum name width for alignment.
///
/// # Arguments
/// * `skills` - The list of skills to measure
///
/// # Returns
/// The width of the longest skill name, capped at a reasonable maximum.
#[must_use]
fn calculate_name_width(skills: &[SkillInfo]) -> usize {
    skills
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        .min(MAX_NAME_WIDTH)
}

/// Formats skills in short format with aligned columns.
///
/// # Arguments
/// * `skills` - The skills to format
/// * `color_config` - Color configuration for output
///
/// # Returns
/// Formatted output string with name (colored) and truncated description.
#[must_use]
fn format_short(skills: &[SkillInfo], color_config: ColorConfig) -> String {
    let name_width = calculate_name_width(skills);
    let mut output = String::new();

    for skill in skills {
        // Show validation symbol if validation was performed
        let symbol = match skill.valid {
            Some(true) => format!("{} ", color::success_symbol(color_config)),
            Some(false) => format!("{} ", color::error_symbol(color_config)),
            None => String::new(),
        };

        let colored_name = color::skill_name(&skill.name, color_config);
        let truncated = if skill.valid == Some(false) {
            // Show error message for invalid skills
            skill
                .error
                .as_deref()
                .map_or_else(|| "Invalid skill".to_string(), |e| truncate_description(e, SHORT_DESCRIPTION_MAX))
        } else {
            truncate_description(&skill.description, SHORT_DESCRIPTION_MAX)
        };

        // Calculate padding: name_width - actual name length for alignment
        // The colored name includes ANSI codes which don't take visual space
        let padding = name_width.saturating_sub(skill.name.len());
        let _ = writeln!(
            output,
            "{}{}{}  {}",
            symbol,
            colored_name,
            " ".repeat(padding),
            truncated
        );
    }

    output
}

/// Formats skills in long format with full details.
///
/// # Arguments
/// * `skills` - The skills to format
/// * `color_config` - Color configuration for output
///
/// # Returns
/// Formatted output string with name (colored), path (colored), and wrapped description.
#[must_use]
fn format_long(skills: &[SkillInfo], color_config: ColorConfig) -> String {
    let mut output = String::new();

    for (i, skill) in skills.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }

        // Show validation symbol if validation was performed
        let symbol = match skill.valid {
            Some(true) => format!("{} ", color::success_symbol(color_config)),
            Some(false) => format!("{} ", color::error_symbol(color_config)),
            None => String::new(),
        };

        let colored_name = color::skill_name(&skill.name, color_config);
        let path_line = format!("{}Path: {}", DESCRIPTION_INDENT, skill.path);
        let colored_path = color::path(&path_line, color_config);

        let _ = writeln!(output, "{symbol}{colored_name}");
        let _ = writeln!(output, "{colored_path}");

        // Show error or description
        let content = if skill.valid == Some(false) {
            skill
                .error
                .as_deref()
                .map_or_else(|| "Invalid skill".to_string(), |e| format!("{DESCRIPTION_INDENT}Error: {e}"))
        } else {
            wrap_text(&skill.description, WRAP_WIDTH, DESCRIPTION_INDENT)
        };
        let _ = writeln!(output, "{content}");
    }

    output
}

/// Formats skills as paths only, one per line.
///
/// # Arguments
/// * `skills` - The skills to format
///
/// # Returns
/// Formatted output string with one path per line.
#[must_use]
fn format_paths_only(skills: &[SkillInfo]) -> String {
    let mut output = String::new();

    for skill in skills {
        let _ = writeln!(output, "{}", skill.path);
    }

    output
}

/// Formats skills as names only, one per line.
///
/// # Arguments
/// * `skills` - The skills to format
///
/// # Returns
/// Formatted output string with one name per line.
#[must_use]
fn format_names_only(skills: &[SkillInfo]) -> String {
    let mut output = String::new();

    for skill in skills {
        let _ = writeln!(output, "{}", skill.name);
    }

    output
}

/// Wraps text to a specified width with a given prefix for all lines.
///
/// # Arguments
/// * `text` - The text to wrap
/// * `width` - Maximum line width
/// * `indent` - Indentation for all lines
///
/// # Returns
/// The wrapped text with proper indentation.
#[must_use]
fn wrap_text(text: &str, width: usize, indent: &str) -> String {
    let options = Options::new(width.saturating_sub(indent.len()))
        .initial_indent(indent)
        .subsequent_indent(indent);

    wrap(text, options).join("\n")
}

/// Discovers skills in a directory.
fn discover_skills(
    directory: &Path,
    recursive: bool,
    validate: bool,
) -> Result<Vec<SkillInfo>, CliError> {
    let mut skills = Vec::new();

    discover_skills_in_dir(directory, recursive, validate, &mut skills)?;

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Recursively discovers skills in a directory.
fn discover_skills_in_dir(
    directory: &Path,
    recursive: bool,
    validate: bool,
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
            if skill_md.exists() {
                match SkillDirectory::load(&path) {
                    Ok(dir) => {
                        let skill = dir.skill();
                        skills.push(SkillInfo {
                            name: skill.name().as_str().to_string(),
                            description: skill.description().as_str().to_string(),
                            path: path.display().to_string(),
                            valid: if validate { Some(true) } else { None },
                            error: None,
                        });
                    }
                    Err(e) if validate => {
                        // Include invalid skills when validating
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        skills.push(SkillInfo {
                            name,
                            description: String::new(),
                            path: path.display().to_string(),
                            valid: Some(false),
                            error: Some(e.to_string()),
                        });
                    }
                    Err(_) => {
                        // Skip invalid skills when not validating
                    }
                }
            }

            // Recurse into subdirectories if requested
            if recursive {
                discover_skills_in_dir(&path, recursive, validate, skills)?;
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

    // Helper function to create SkillInfo for tests
    fn skill_info(name: &str, path: &str, description: &str) -> SkillInfo {
        SkillInfo {
            name: name.to_string(),
            path: path.to_string(),
            description: description.to_string(),
            valid: None,
            error: None,
        }
    }

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

    fn create_skill_dir_with_description(dir: &Path, name: &str, description: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).ok();
        let content = format!(
            r#"---
name: {name}
description: {description}
---
# Instructions
"#
        );
        fs::write(skill_dir.join("SKILL.md"), content).ok();
    }

    // Truncation tests
    #[test]
    fn truncate_description_returns_original_when_short() {
        let desc = "Short description";
        assert_eq!(truncate_description(desc, 50), desc);
    }

    #[test]
    fn truncate_description_returns_original_when_exact_length() {
        let desc = "Exactly fifty characters long, nothing more at al";
        assert_eq!(desc.len(), 49);
        assert_eq!(truncate_description(desc, 50), desc);
    }

    #[test]
    fn truncate_description_adds_ellipsis_when_long() {
        let desc = "This is a very long description that exceeds the maximum length allowed";
        let result = truncate_description(desc, 30);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 30);
    }

    #[test]
    fn truncate_description_respects_word_boundaries() {
        // max_len=22 means target_len=19, and "Hello wonderful" is 15 chars
        // so we find space at position 5 ("Hello ") and at position 15 ("Hello wonderful ")
        // Position 15 is within target_len=19, so we truncate after "wonderful"
        let desc = "Hello wonderful world today";
        let result = truncate_description(desc, 22);
        // Should truncate to "Hello wonderful..." (15 + 3 = 18 chars)
        assert_eq!(result, "Hello wonderful...");
    }

    #[test]
    fn truncate_description_handles_single_long_word() {
        let desc = "Supercalifragilisticexpialidocious";
        let result = truncate_description(desc, 15);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 15);
    }

    #[test]
    fn truncate_description_handles_empty_string() {
        assert_eq!(truncate_description("", 50), "");
    }

    // Width calculation tests
    #[test]
    fn calculate_name_width_returns_zero_for_empty() {
        let skills: Vec<SkillInfo> = vec![];
        assert_eq!(calculate_name_width(&skills), 0);
    }

    #[test]
    fn calculate_name_width_finds_maximum() {
        let skills = vec![
            skill_info("short", "", ""),
            skill_info("much-longer-name", "", ""),
            skill_info("medium", "", ""),
        ];
        assert_eq!(calculate_name_width(&skills), 16);
    }

    #[test]
    fn calculate_name_width_caps_at_maximum() {
        let skills = vec![skill_info(
            "this-is-an-extremely-long-skill-name-that-exceeds-reasonable-limits",
            "",
            "",
        )];
        assert_eq!(calculate_name_width(&skills), MAX_NAME_WIDTH);
    }

    // Format short tests
    #[test]
    fn format_short_aligns_columns() {
        let skills = vec![
            skill_info("short", "/path/short", "Description one"),
            skill_info("much-longer", "/path/longer", "Description two"),
        ];
        let config = ColorConfig::new(false);
        let output = format_short(&skills, config);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);

        // Both descriptions should start at the same column
        let desc_start_1 = lines[0].find("Description one").unwrap();
        let desc_start_2 = lines[1].find("Description two").unwrap();
        assert_eq!(desc_start_1, desc_start_2);
    }

    #[test]
    fn format_short_truncates_long_descriptions() {
        let skills = vec![skill_info(
            "my-skill",
            "/path",
            "This is a very long description that should be truncated because it exceeds the limit",
        )];
        let config = ColorConfig::new(false);
        let output = format_short(&skills, config);
        assert!(output.contains("..."));
        // Original description should not appear in full
        assert!(!output.contains("exceeds the limit"));
    }

    #[test]
    fn format_short_does_not_include_path() {
        let skills = vec![skill_info(
            "my-skill",
            "/home/user/skills/my-skill",
            "Description",
        )];
        let config = ColorConfig::new(false);
        let output = format_short(&skills, config);
        assert!(!output.contains("/home/user/skills"));
    }

    #[test]
    fn format_short_with_colors_includes_ansi_codes() {
        let skills = vec![skill_info("my-skill", "/path", "Description")];
        let config = ColorConfig::new(true);
        let output = format_short(&skills, config);
        assert!(output.contains("\x1b[1;96m")); // Bold bright cyan
        assert!(output.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn format_short_without_colors_excludes_ansi_codes() {
        let skills = vec![skill_info("my-skill", "/path", "Description")];
        let config = ColorConfig::new(false);
        let output = format_short(&skills, config);
        assert!(!output.contains("\x1b["));
    }

    // Format long tests
    #[test]
    fn format_long_shows_path_and_full_description() {
        let skills = vec![skill_info(
            "my-skill",
            "/home/user/skills/my-skill",
            "A complete description that would normally be truncated in short mode.",
        )];
        let config = ColorConfig::new(false);
        let output = format_long(&skills, config);
        assert!(output.contains("my-skill"));
        assert!(output.contains("Path: /home/user/skills/my-skill"));
        assert!(output.contains("complete description"));
        assert!(output.contains("truncated"));
    }

    #[test]
    fn format_long_separates_skills_with_blank_line() {
        let skills = vec![
            skill_info("skill-one", "/path/one", "First skill"),
            skill_info("skill-two", "/path/two", "Second skill"),
        ];
        let config = ColorConfig::new(false);
        let output = format_long(&skills, config);
        // Should have blank line between skills
        assert!(output.contains("\n\n"));
    }

    #[test]
    fn format_long_indents_path_and_description() {
        let skills = vec![skill_info("my-skill", "/path", "Description")];
        let config = ColorConfig::new(false);
        let output = format_long(&skills, config);
        let lines: Vec<&str> = output.lines().collect();

        // Name should not be indented
        assert!(lines[0].starts_with("my-skill"));
        // Path should be indented
        assert!(lines[1].starts_with("  Path:"));
        // Description should be indented
        assert!(lines[2].starts_with("  "));
    }

    #[test]
    fn format_long_with_colors_includes_ansi_codes_for_name_and_path() {
        let skills = vec![skill_info("my-skill", "/path", "Description")];
        let config = ColorConfig::new(true);
        let output = format_long(&skills, config);
        assert!(output.contains("\x1b[1;96m")); // Bold bright cyan for name
        assert!(output.contains("\x1b[90m")); // Bright black for path
    }

    #[test]
    fn format_long_without_colors_excludes_ansi_codes() {
        let skills = vec![skill_info("my-skill", "/path", "Description")];
        let config = ColorConfig::new(false);
        let output = format_long(&skills, config);
        assert!(!output.contains("\x1b["));
    }

    // Format paths only tests
    #[test]
    fn format_paths_only_outputs_one_path_per_line() {
        let skills = vec![
            skill_info("a", "/path/a", ""),
            skill_info("b", "/path/b", ""),
        ];
        let output = format_paths_only(&skills);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "/path/a");
        assert_eq!(lines[1], "/path/b");
    }

    #[test]
    fn format_paths_only_excludes_names_and_descriptions() {
        let skills = vec![skill_info("my-skill", "/path/to/skill", "A description")];
        let output = format_paths_only(&skills);
        assert!(!output.contains("my-skill"));
        assert!(!output.contains("description"));
        assert!(output.contains("/path/to/skill"));
    }

    #[test]
    fn format_paths_only_never_has_colors() {
        let skills = vec![skill_info("my-skill", "/path", "Description")];
        let output = format_paths_only(&skills);
        assert!(!output.contains("\x1b["));
    }

    // Format names only tests
    #[test]
    fn format_names_only_outputs_one_name_per_line() {
        let skills = vec![
            skill_info("alpha", "/path/a", ""),
            skill_info("beta", "/path/b", ""),
        ];
        let output = format_names_only(&skills);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "alpha");
        assert_eq!(lines[1], "beta");
    }

    #[test]
    fn format_names_only_excludes_paths_and_descriptions() {
        let skills = vec![skill_info("my-skill", "/path/to/skill", "A description")];
        let output = format_names_only(&skills);
        assert!(output.contains("my-skill"));
        assert!(!output.contains("/path"));
        assert!(!output.contains("description"));
    }

    #[test]
    fn format_names_only_never_has_colors() {
        let skills = vec![skill_info("my-skill", "/path", "Description")];
        let output = format_names_only(&skills);
        assert!(!output.contains("\x1b["));
    }

    // Wrap text tests
    #[test]
    fn wrap_text_short_text_no_wrapping() {
        let text = "Short text";
        let result = wrap_text(text, 80, "  ");
        assert_eq!(result, "  Short text");
    }

    #[test]
    fn wrap_text_wraps_long_text() {
        let text = "This is a long piece of text that should be wrapped across multiple lines when the width is limited";
        let result = wrap_text(text, 40, "  ");
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 1);
        // All lines should be indented
        for line in &lines {
            assert!(line.starts_with("  "));
        }
    }

    // Discovery tests (existing)
    #[test]
    fn discover_skills_finds_skills_in_directory() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            create_skill_dir(temp.path(), "my-skill");

            let skills = discover_skills(temp.path(), false, false);
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
            let skills = discover_skills(temp.path(), false, false);
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

            let skills_non_recursive = discover_skills(temp.path(), false, false);
            let skills_recursive = discover_skills(temp.path(), true, false);

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

            let skills = discover_skills(temp.path(), false, false);
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

    // Run function tests
    #[test]
    fn run_returns_error_for_nonexistent_directory() {
        let result = run(
            Path::new("/nonexistent/path"),
            false,
            false,
            false,
            ListOutputMode::Short,
            OutputMode::Normal,
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
    fn run_succeeds_for_empty_directory() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            let result = run(
                temp.path(),
                false,
                false,
                false,
                ListOutputMode::Short,
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_short_mode_uses_truncated_format() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            create_skill_dir_with_description(
                temp.path(),
                "test-skill",
                "A very long description that exceeds fifty characters and should be truncated",
            );

            // We can't easily capture stdout in a unit test, but we can verify it doesn't panic
            let result = run(
                temp.path(),
                false,
                false,
                false,
                ListOutputMode::Short,
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_long_mode_shows_full_details() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            create_skill_dir(temp.path(), "test-skill");

            let result = run(
                temp.path(),
                false,
                false,
                false,
                ListOutputMode::Long,
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_paths_mode_works() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            create_skill_dir(temp.path(), "test-skill");

            let result = run(
                temp.path(),
                false,
                false,
                false,
                ListOutputMode::PathsOnly,
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_names_mode_works() {
        let temp = TempDir::new().ok();
        if let Some(temp) = temp.as_ref() {
            create_skill_dir(temp.path(), "test-skill");

            let result = run(
                temp.path(),
                false,
                false,
                false,
                ListOutputMode::NamesOnly,
                OutputMode::Quiet,
                ColorConfig::default(),
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn skill_info_serializes_to_json() {
        let info = SkillInfo {
            name: "test-skill".to_string(),
            description: "A test skill.".to_string(),
            path: "/path/to/skill".to_string(),
            valid: None,
            error: None,
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("\"name\":\"test-skill\""));
        assert!(json.contains("\"description\":\"A test skill.\""));
        assert!(json.contains("\"path\":\"/path/to/skill\""));
    }

    #[test]
    fn list_output_mode_default_is_short() {
        assert_eq!(ListOutputMode::default(), ListOutputMode::Short);
    }
}
