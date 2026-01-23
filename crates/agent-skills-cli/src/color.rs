//! Terminal color support for CLI output.
//!
//! Provides ANSI color codes and helper functions for colorizing output.
//! Colors are disabled when any of the following are true:
//! - The `--no-color` CLI flag is set
//! - The `NO_COLOR` environment variable is set (any value, per <https://no-color.org/>)
//! - The `TERM` environment variable equals "dumb"
//! - stdout is not a TTY

use std::io::IsTerminal;

/// ANSI escape code for bold bright cyan (skill names).
pub const BOLD_BRIGHT_CYAN: &str = "\x1b[1;96m";

/// ANSI escape code for bright black/gray (paths, summary).
pub const BRIGHT_BLACK: &str = "\x1b[90m";

/// ANSI escape code for bright red (error labels).
pub const BRIGHT_RED: &str = "\x1b[91m";

/// ANSI escape code for bright green (success indicators).
pub const BRIGHT_GREEN: &str = "\x1b[92m";

/// ANSI escape code to reset all formatting.
pub const RESET: &str = "\x1b[0m";

/// Unicode checkmark symbol.
pub const CHECKMARK: &str = "✓";

/// Unicode cross symbol.
pub const CROSS: &str = "✗";

/// Configuration for terminal color output.
///
/// Controls whether ANSI color codes are emitted in output.
/// Colors are disabled when any of the following are true:
/// - The `--no-color` CLI flag is set
/// - The `NO_COLOR` environment variable is set (any value, per <https://no-color.org/>)
/// - The `TERM` environment variable equals "dumb"
/// - stdout is not a TTY
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorConfig {
    enabled: bool,
}

impl ColorConfig {
    /// Creates a new color configuration.
    ///
    /// # Arguments
    /// * `enabled` - Whether colors should be used
    #[must_use]
    pub const fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether colors are enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    /// Detects whether colors should be enabled based on TTY, flags, and environment.
    ///
    /// Colors are disabled when any of the following are true:
    /// - The `--no-color` CLI flag is set
    /// - The `NO_COLOR` environment variable is set (any value, per <https://no-color.org/>)
    /// - The `TERM` environment variable equals "dumb"
    /// - stdout is not a TTY
    ///
    /// # Arguments
    /// * `no_color_flag` - Whether the `--no-color` CLI flag was set
    ///
    /// # Returns
    /// A `ColorConfig` with colors enabled only if all checks pass.
    #[must_use]
    pub fn detect(no_color_flag: bool) -> Self {
        let no_color_env = std::env::var("NO_COLOR").is_ok();
        let term_dumb = std::env::var("TERM")
            .map(|t| t == "dumb")
            .unwrap_or(false);
        let enabled =
            !no_color_flag && !no_color_env && !term_dumb && std::io::stdout().is_terminal();
        Self::new(enabled)
    }
}

/// Formats text with the skill name color (bold bright cyan).
///
/// # Arguments
/// * `text` - The text to colorize
/// * `config` - Color configuration
///
/// # Returns
/// The text with ANSI codes if colors are enabled, otherwise unchanged.
#[must_use]
pub fn skill_name(text: &str, config: ColorConfig) -> String {
    if config.is_enabled() {
        format!("{BOLD_BRIGHT_CYAN}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Formats text with the path color (bright black/gray).
///
/// # Arguments
/// * `text` - The text to colorize
/// * `config` - Color configuration
///
/// # Returns
/// The text with ANSI codes if colors are enabled, otherwise unchanged.
#[must_use]
pub fn path(text: &str, config: ColorConfig) -> String {
    if config.is_enabled() {
        format!("{BRIGHT_BLACK}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Formats text with the summary color (bright black/gray).
///
/// # Arguments
/// * `text` - The text to colorize
/// * `config` - Color configuration
///
/// # Returns
/// The text with ANSI codes if colors are enabled, otherwise unchanged.
#[must_use]
pub fn summary(text: &str, config: ColorConfig) -> String {
    if config.is_enabled() {
        format!("{BRIGHT_BLACK}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Formats text with the error label color (bright red).
///
/// # Arguments
/// * `text` - The text to colorize
/// * `config` - Color configuration
///
/// # Returns
/// The text with ANSI codes if colors are enabled, otherwise unchanged.
#[must_use]
pub fn error_label(text: &str, config: ColorConfig) -> String {
    if config.is_enabled() {
        format!("{BRIGHT_RED}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Returns a green checkmark symbol for success indicators.
///
/// # Arguments
/// * `config` - Color configuration
///
/// # Returns
/// A green checkmark if colors are enabled, otherwise a plain checkmark.
#[must_use]
pub fn success_symbol(config: ColorConfig) -> String {
    if config.is_enabled() {
        format!("{BRIGHT_GREEN}{CHECKMARK}{RESET}")
    } else {
        CHECKMARK.to_string()
    }
}

/// Returns a red cross symbol for error indicators.
///
/// # Arguments
/// * `config` - Color configuration
///
/// # Returns
/// A red cross if colors are enabled, otherwise a plain cross.
#[must_use]
pub fn error_symbol(config: ColorConfig) -> String {
    if config.is_enabled() {
        format!("{BRIGHT_RED}{CROSS}{RESET}")
    } else {
        CROSS.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_config_new_enabled() {
        let config = ColorConfig::new(true);
        assert!(config.is_enabled());
    }

    #[test]
    fn color_config_new_disabled() {
        let config = ColorConfig::new(false);
        assert!(!config.is_enabled());
    }

    #[test]
    fn color_config_default_is_disabled() {
        let config = ColorConfig::default();
        assert!(!config.is_enabled());
    }

    #[test]
    fn skill_name_with_colors_enabled() {
        let config = ColorConfig::new(true);
        let result = skill_name("test-skill", config);
        assert_eq!(result, "\x1b[1;96mtest-skill\x1b[0m");
    }

    #[test]
    fn skill_name_with_colors_disabled() {
        let config = ColorConfig::new(false);
        let result = skill_name("test-skill", config);
        assert_eq!(result, "test-skill");
    }

    #[test]
    fn path_with_colors_enabled() {
        let config = ColorConfig::new(true);
        let result = path("/some/path", config);
        assert_eq!(result, "\x1b[90m/some/path\x1b[0m");
    }

    #[test]
    fn path_with_colors_disabled() {
        let config = ColorConfig::new(false);
        let result = path("/some/path", config);
        assert_eq!(result, "/some/path");
    }

    #[test]
    fn summary_with_colors_enabled() {
        let config = ColorConfig::new(true);
        let result = summary("Found 5 skill(s)", config);
        assert_eq!(result, "\x1b[90mFound 5 skill(s)\x1b[0m");
    }

    #[test]
    fn summary_with_colors_disabled() {
        let config = ColorConfig::new(false);
        let result = summary("Found 5 skill(s)", config);
        assert_eq!(result, "Found 5 skill(s)");
    }

    #[test]
    fn error_label_with_colors_enabled() {
        let config = ColorConfig::new(true);
        let result = error_label("Error:", config);
        assert_eq!(result, "\x1b[91mError:\x1b[0m");
    }

    #[test]
    fn error_label_with_colors_disabled() {
        let config = ColorConfig::new(false);
        let result = error_label("Error:", config);
        assert_eq!(result, "Error:");
    }

    #[test]
    fn success_symbol_with_colors_enabled() {
        let config = ColorConfig::new(true);
        let result = success_symbol(config);
        assert_eq!(result, "\x1b[92m✓\x1b[0m");
    }

    #[test]
    fn success_symbol_with_colors_disabled() {
        let config = ColorConfig::new(false);
        let result = success_symbol(config);
        assert_eq!(result, "✓");
    }

    #[test]
    fn error_symbol_with_colors_enabled() {
        let config = ColorConfig::new(true);
        let result = error_symbol(config);
        assert_eq!(result, "\x1b[91m✗\x1b[0m");
    }

    #[test]
    fn error_symbol_with_colors_disabled() {
        let config = ColorConfig::new(false);
        let result = error_symbol(config);
        assert_eq!(result, "✗");
    }

    #[test]
    fn detect_disables_colors_when_no_color_flag_set() {
        // When the --no-color flag is set, colors should be disabled
        // regardless of TTY status (which is false in tests)
        let config = ColorConfig::detect(true);
        assert!(!config.is_enabled());
    }

    #[test]
    fn detect_disables_colors_in_non_tty() {
        // In test environment, stdout is typically not a TTY
        // So even without --no-color flag, colors should be disabled
        let config = ColorConfig::detect(false);
        // Note: This test depends on the test environment not being a TTY
        // If it is a TTY, colors would be enabled. The test verifies
        // that the detect function works correctly for non-TTY environments.
        assert!(!config.is_enabled());
    }
}
