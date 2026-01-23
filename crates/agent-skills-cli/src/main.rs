//! Agent Skills CLI.
//!
//! A command-line tool for validating and working with Agent Skills.
//!
//! # Commands
//!
//! - `validate <path>` - Validate a skill directory
//! - `read-properties <path>` - Read skill properties as JSON
//! - `to-prompt <paths...>` - Generate XML block for agent prompts

mod commands;
mod error;
pub mod xml;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use error::CliError;

#[derive(Parser)]
#[command(name = "agent-skills")]
#[command(about = "CLI for validating and working with Agent Skills")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a skill directory
    Validate {
        /// Path to skill directory or SKILL.md file
        skill_path: PathBuf,
    },
    /// Read skill properties as JSON
    #[command(name = "read-properties")]
    ReadProperties {
        /// Path to skill directory or SKILL.md file
        skill_path: PathBuf,
    },
    /// Generate `available_skills` XML block for agent prompts
    #[command(name = "to-prompt")]
    ToPrompt {
        /// Paths to skill directories or SKILL.md files
        #[arg(required = true)]
        skill_paths: Vec<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = run_command(&cli.command);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_command(command: &Commands) -> Result<(), CliError> {
    match command {
        Commands::Validate { skill_path } => commands::validate::run(skill_path),
        Commands::ReadProperties { skill_path } => commands::read_properties::run(skill_path),
        Commands::ToPrompt { skill_paths } => commands::to_prompt::run(skill_paths),
    }
}
