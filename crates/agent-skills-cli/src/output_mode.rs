//! Output mode configuration for the CLI.

/// Output mode controlling verbosity and format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    /// Normal output (default).
    #[default]
    Normal,
    /// Quiet mode - suppress informational messages.
    Quiet,
    /// Verbose mode - show additional progress information.
    Verbose,
    /// JSON mode - output structured JSON for scripting.
    Json,
}

impl OutputMode {
    /// Returns true if informational messages should be shown.
    #[must_use]
    pub const fn show_info(self) -> bool {
        matches!(self, Self::Normal | Self::Verbose)
    }

    /// Returns true if progress messages should be shown.
    #[must_use]
    pub const fn show_progress(self) -> bool {
        matches!(self, Self::Verbose)
    }

    /// Returns true if output should be JSON formatted.
    #[must_use]
    pub const fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_mode_default_is_normal() {
        assert_eq!(OutputMode::default(), OutputMode::Normal);
    }

    #[test]
    fn output_mode_normal_shows_info() {
        assert!(OutputMode::Normal.show_info());
    }

    #[test]
    fn output_mode_normal_does_not_show_progress() {
        assert!(!OutputMode::Normal.show_progress());
    }

    #[test]
    fn output_mode_normal_is_not_json() {
        assert!(!OutputMode::Normal.is_json());
    }

    #[test]
    fn output_mode_quiet_does_not_show_info() {
        assert!(!OutputMode::Quiet.show_info());
    }

    #[test]
    fn output_mode_quiet_does_not_show_progress() {
        assert!(!OutputMode::Quiet.show_progress());
    }

    #[test]
    fn output_mode_quiet_is_not_json() {
        assert!(!OutputMode::Quiet.is_json());
    }

    #[test]
    fn output_mode_verbose_shows_info() {
        assert!(OutputMode::Verbose.show_info());
    }

    #[test]
    fn output_mode_verbose_shows_progress() {
        assert!(OutputMode::Verbose.show_progress());
    }

    #[test]
    fn output_mode_verbose_is_not_json() {
        assert!(!OutputMode::Verbose.is_json());
    }

    #[test]
    fn output_mode_json_does_not_show_info() {
        assert!(!OutputMode::Json.show_info());
    }

    #[test]
    fn output_mode_json_does_not_show_progress() {
        assert!(!OutputMode::Json.show_progress());
    }

    #[test]
    fn output_mode_json_is_json() {
        assert!(OutputMode::Json.is_json());
    }
}
