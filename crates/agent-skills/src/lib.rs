//! Parse, validate, and work with Agent Skills.
//!
//! This crate provides types and functions for working with Agent Skills
//! as defined by the [Agent Skills specification](https://agentskills.io).
//!
//! # Overview
//!
//! Agent Skills are directories containing a `SKILL.md` file with YAML frontmatter
//! and markdown instructions. This crate handles:
//!
//! - Parsing `SKILL.md` files (frontmatter + markdown body)
//! - Validating skills against the specification
//! - Loading skills from directories
//! - Accessing skill metadata, instructions, and referenced files
//! - **Security verification** via agent-uri identity and PASETO attestation
//!
//! # Quick Start
//!
//! ## Parsing a SKILL.md file
//!
//! ```
//! use agent_skills::Skill;
//!
//! let content = r#"---
//! name: my-skill
//! description: Does something useful.
//! ---
//! # Instructions
//!
//! Follow these steps...
//! "#;
//!
//! let skill = Skill::parse(content).unwrap();
//! assert_eq!(skill.name().as_str(), "my-skill");
//! assert!(skill.body().contains("# Instructions"));
//! ```
//!
//! ## Loading a skill from a directory
//!
//! ```no_run
//! use agent_skills::SkillDirectory;
//! use std::path::Path;
//!
//! let dir = SkillDirectory::load(Path::new("./my-skill")).unwrap();
//! println!("Loaded skill: {}", dir.skill().name());
//!
//! // Access optional directories
//! if dir.has_scripts() {
//!     for script in dir.scripts().unwrap() {
//!         println!("Script: {}", script.display());
//!     }
//! }
//! ```
//!
//! ## Creating skills programmatically
//!
//! ```
//! use agent_skills::{Skill, Frontmatter, SkillName, SkillDescription, Metadata};
//!
//! let name = SkillName::new("my-skill").unwrap();
//! let desc = SkillDescription::new("Does something useful.").unwrap();
//!
//! let frontmatter = Frontmatter::builder(name, desc)
//!     .license("MIT")
//!     .metadata(Metadata::from_pairs([("author", "example")]))
//!     .build();
//!
//! let skill = Skill::new(frontmatter, "# Instructions\n\nDo this.");
//! ```
//!
//! # Types
//!
//! - [`Skill`] - A complete skill with frontmatter and body
//! - [`Frontmatter`] - The YAML frontmatter header
//! - [`SkillName`] - A validated skill name
//! - [`SkillDescription`] - A validated skill description
//! - [`Compatibility`] - Environment compatibility requirements
//! - [`Metadata`] - Arbitrary key-value metadata
//! - [`AllowedTools`] - Pre-approved tool list (experimental)
//! - [`SkillDirectory`] - A loaded skill directory with file access
//!
//! # Security Module
//!
//! The [`security`] module provides cryptographic verification of skills:
//!
//! - [`security::SecureSkillRegistry`] - Registry with attestation verification
//! - [`security::VerifiedSkill`] - Skill with verified identity and capabilities
//! - [`security::TrustTier`] - Trust levels for capability-based access control
//!
//! See the [`security`] module documentation for details.
//!
//! # Errors
//!
//! - [`ParseError`] - Errors when parsing SKILL.md content
//! - [`LoadError`] - Errors when loading skill directories
//! - [`SkillNameError`] - Invalid skill name
//! - [`SkillDescriptionError`] - Invalid skill description
//! - [`CompatibilityError`] - Invalid compatibility string
//! - [`security::SkillSecurityError`] - Security verification errors

mod allowed_tools;
mod compatibility;
mod description;
mod error;
mod frontmatter;
mod loader;
mod metadata;
mod name;
pub mod security;
mod skill;

// Re-export main types
pub use allowed_tools::AllowedTools;
pub use compatibility::{Compatibility, CompatibilityError};
pub use description::{SkillDescription, SkillDescriptionError};
pub use error::{LoadError, ParseError};
pub use frontmatter::{Frontmatter, FrontmatterBuilder};
pub use loader::SkillDirectory;
pub use metadata::Metadata;
pub use name::{SkillName, SkillNameError};
pub use skill::Skill;
