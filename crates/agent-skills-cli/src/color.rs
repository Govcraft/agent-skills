//! Terminal color support for CLI output.
//!
//! Provides ANSI color codes and helper functions for colorizing output.
//! Colors are only enabled when stdout is a TTY and `--no-color` is not set.

use std::io::IsTerminal;

/// ANSI escape code for bold bright cyan (skill names).
pub const BOLD_BRIGHT_CYAN: &str = "\x1b[1;96m";

/// ANSI escape code for bright black/gray (paths, summary).
pub const BRIGHT_BLACK: &str = "\x1b[90m";

/// ANSI escape code to reset all formatting.
pub const RESET: &str = "\x1b[0m";

/// Configuration for terminal color output.
///
/// Controls whether ANSI color codes are emitted in output.
/// Colors are enabled only when stdout is a TTY and `--no-color` is not set.
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

    /// Detects whether colors should be enabled based on TTY and flag.
    ///
    /// # Arguments
    /// * `no_color_flag` - Whether the `--no-color` CLI flag was set
    ///
    /// # Returns
    /// A `ColorConfig` with colors enabled if stdout is a TTY and `--no-color` is false.
    #[must_use]
    pub fn detect(no_color_flag: bool) -> Self {
        let enabled = !no_color_flag && std::io::stdout().is_terminal();
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
}
