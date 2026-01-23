//! Agent Skills CLI.
//!
//! A command-line tool for validating and working with Agent Skills.
//!
//! # Commands
//!
//! - `validate <path>` - Validate a skill directory (alias: `check`)
//! - `read-properties <path>` - Read skill properties as JSON/YAML/TOML (alias: `props`)
//! - `to-prompt <paths...>` - Generate XML block for agent prompts (alias: `prompt`)
//! - `list <directory>` - List skills in a directory (alias: `ls`)

mod color;
mod commands;
mod error;
mod exit_code;
mod output_format;
mod output_mode;
pub mod xml;

use std::path::PathBuf;
use std::process::ExitCode as StdExitCode;

use clap::{Parser, Subcommand};

use color::ColorConfig;
use commands::list::ListOutputMode;
use error::CliError;
use exit_code::ExitCode;
use output_format::OutputFormat;
use output_mode::OutputMode;

const DOCUMENTATION_URL: &str = "https://agentskills.io";

/// Verbosity level for CLI output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Show extra progress information.
    Verbose,
    /// Normal output level.
    #[default]
    Normal,
    /// Suppress informational output.
    Quiet,
}

/// CLI flags parsed into semantic types.
///
/// This struct groups related options to avoid excessive boolean fields.
#[derive(Debug, Clone, Copy, Default, clap::Args)]
pub struct OutputOptions {
    /// Increase output verbosity (show progress information)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress informational output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
}

impl OutputOptions {
    /// Determines the verbosity level from flags.
    #[must_use]
    pub const fn verbosity(&self) -> Verbosity {
        if self.quiet {
            Verbosity::Quiet
        } else if self.verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        }
    }

    /// Returns the color configuration based on flags and TTY detection.
    #[must_use]
    pub fn color_config(&self) -> ColorConfig {
        ColorConfig::detect(self.no_color)
    }
}

/// Error output format options.
#[derive(Debug, Clone, Copy, Default, clap::Args)]
pub struct ErrorOptions {
    /// Output errors as JSON for scripting
    #[arg(long, global = true)]
    json: bool,
}

impl ErrorOptions {
    /// Returns whether JSON output is enabled.
    #[must_use]
    pub const fn is_json(&self) -> bool {
        self.json
    }
}

#[derive(Parser)]
#[command(name = "agent-skills")]
#[command(about = "CLI for validating and working with Agent Skills")]
#[command(version)]
#[command(infer_subcommands = true)]
#[command(after_help = format!("Environment:\n  NO_COLOR     Disable colored output when set (any value)\n  TERM=dumb    Disable colored output when set\n\nDocumentation: {DOCUMENTATION_URL}"))]
struct Cli {
    #[command(flatten)]
    output: OutputOptions,

    #[command(flatten)]
    error: ErrorOptions,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Determines the output mode from flags.
    const fn output_mode(&self) -> OutputMode {
        if self.error.is_json() {
            OutputMode::Json
        } else {
            match self.output.verbosity() {
                Verbosity::Quiet => OutputMode::Quiet,
                Verbosity::Verbose => OutputMode::Verbose,
                Verbosity::Normal => OutputMode::Normal,
            }
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a skill directory
    #[command(visible_alias = "check")]
    #[command(after_help = r#"Examples:
  agent-skills validate ./my-skill
  agent-skills validate ./my-skill/SKILL.md
  find ./skills -name SKILL.md | agent-skills validate -
  agent-skills check ./my-skill"#)]
    Validate {
        /// Path to skill directory or SKILL.md file, or "-" to read from stdin
        #[arg(default_value = ".")]
        skill_path: String,
    },

    /// Read skill properties as JSON/YAML/TOML
    #[command(name = "read-properties", visible_alias = "props")]
    #[command(after_help = r#"Examples:
  agent-skills read-properties ./my-skill
  agent-skills props ./my-skill --format yaml
  agent-skills props ./my-skill --format json --compact"#)]
    ReadProperties {
        /// Path to skill directory or SKILL.md file, or "-" to read from stdin
        #[arg(default_value = ".")]
        skill_path: String,

        /// Output format
        #[arg(short, long, default_value = "json", value_parser = clap::value_parser!(OutputFormat))]
        format: OutputFormat,

        /// Compact output (no pretty printing, JSON only)
        #[arg(long)]
        compact: bool,
    },

    /// Generate `available_skills` XML block for agent prompts
    #[command(name = "to-prompt", visible_alias = "prompt")]
    #[command(after_help = r#"Examples:
  agent-skills to-prompt ./skill-one ./skill-two
  agent-skills prompt ./skills/*
  find ./skills -name SKILL.md | agent-skills to-prompt -"#)]
    ToPrompt {
        /// Paths to skill directories or SKILL.md files, or "-" to read from stdin
        #[arg(required = true)]
        skill_paths: Vec<String>,
    },

    /// List skills in a directory
    #[command(visible_alias = "ls")]
    #[command(after_help = r#"Examples:
  agent-skills list ./skills
  agent-skills list --long ./skills
  agent-skills list --check ./skills
  agent-skills list --failed ./skills
  agent-skills list --paths ./skills | xargs -I{} ls {}
  agent-skills list --names ./skills
  agent-skills ls -r ./
  agent-skills --json list ./skills"#)]
    List {
        /// Directory to search for skills
        #[arg(default_value = ".")]
        directory: PathBuf,

        /// Search recursively
        #[arg(short, long)]
        recursive: bool,

        /// Validate each skill and show ✓/✗ status
        #[arg(short, long)]
        check: bool,

        /// Show only skills that fail validation (implies --check)
        #[arg(short, long)]
        failed: bool,

        /// Show full details (path and complete description)
        #[arg(short, long, conflicts_with_all = ["paths", "names"])]
        long: bool,

        /// Show only paths (for scripting)
        #[arg(short, long, conflicts_with_all = ["long", "names"])]
        paths: bool,

        /// Show only names (for scripting)
        #[arg(short, long, conflicts_with_all = ["long", "paths"])]
        names: bool,
    },
}

fn main() -> StdExitCode {
    let cli = Cli::parse();
    let output_mode = cli.output_mode();
    let color_config = cli.output.color_config();

    let result = run_command(&cli.command, output_mode, color_config);

    match result {
        Ok(()) => ExitCode::Success.into(),
        Err(e) => {
            let exit_code = ExitCode::from(&e);

            if output_mode.is_json() {
                eprintln!("{}", e.to_json());
            } else {
                let symbol = color::error_symbol(color_config);
                let error_label = color::error_label("Error:", color_config);
                eprintln!("{symbol} {error_label} {e}");
                eprintln!();
                let hint_text = color::summary(&format!("Hint: {}", e.suggestion()), color_config);
                eprintln!("{hint_text}");
            }

            exit_code.into()
        }
    }
}

fn run_command(
    command: &Commands,
    output_mode: OutputMode,
    color_config: ColorConfig,
) -> Result<(), CliError> {
    match command {
        Commands::Validate { skill_path } => {
            commands::validate::run(skill_path, output_mode, color_config)
        }
        Commands::ReadProperties {
            skill_path,
            format,
            compact,
        } => commands::read_properties::run(skill_path, *format, *compact, output_mode),
        Commands::ToPrompt { skill_paths } => {
            commands::to_prompt::run(skill_paths, output_mode, color_config)
        }
        Commands::List {
            directory,
            recursive,
            check,
            failed,
            long,
            paths,
            names,
        } => {
            let list_mode = determine_list_mode(*long, *paths, *names);
            // --failed implies --check
            let validate = *check || *failed;
            // Disable colors for scripting modes
            let effective_color = if list_mode == ListOutputMode::PathsOnly
                || list_mode == ListOutputMode::NamesOnly
            {
                ColorConfig::new(false)
            } else {
                color_config
            };
            commands::list::run(
                directory,
                *recursive,
                validate,
                *failed,
                list_mode,
                output_mode,
                effective_color,
            )
        }
    }
}

/// Determines the list output mode from CLI flags.
///
/// Priority: paths > names > long > short (default)
const fn determine_list_mode(long: bool, paths: bool, names: bool) -> ListOutputMode {
    if paths {
        ListOutputMode::PathsOnly
    } else if names {
        ListOutputMode::NamesOnly
    } else if long {
        ListOutputMode::Long
    } else {
        ListOutputMode::Short
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_options_default_is_normal() {
        let opts = OutputOptions::default();
        assert_eq!(opts.verbosity(), Verbosity::Normal);
    }

    #[test]
    fn output_options_verbose() {
        let opts = OutputOptions {
            verbose: true,
            quiet: false,
            no_color: false,
        };
        assert_eq!(opts.verbosity(), Verbosity::Verbose);
    }

    #[test]
    fn output_options_quiet() {
        let opts = OutputOptions {
            verbose: false,
            quiet: true,
            no_color: false,
        };
        assert_eq!(opts.verbosity(), Verbosity::Quiet);
    }

    #[test]
    fn output_options_quiet_takes_precedence_over_verbose() {
        // Quiet should take precedence if both are set
        let opts = OutputOptions {
            verbose: true,
            quiet: true,
            no_color: false,
        };
        assert_eq!(opts.verbosity(), Verbosity::Quiet);
    }

    #[test]
    fn cli_output_mode_json() {
        let cli = Cli {
            output: OutputOptions::default(),
            error: ErrorOptions { json: true },
            command: Commands::Validate {
                skill_path: ".".to_string(),
            },
        };
        assert_eq!(cli.output_mode(), OutputMode::Json);
    }

    #[test]
    fn cli_output_mode_json_takes_precedence_over_verbose() {
        let cli = Cli {
            output: OutputOptions {
                verbose: true,
                quiet: false,
                no_color: false,
            },
            error: ErrorOptions { json: true },
            command: Commands::Validate {
                skill_path: ".".to_string(),
            },
        };
        assert_eq!(cli.output_mode(), OutputMode::Json);
    }
}
