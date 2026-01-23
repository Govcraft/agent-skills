//! The main Skill type combining frontmatter and markdown body.

use std::collections::HashMap;

use serde::Deserialize;

use crate::allowed_tools::AllowedTools;
use crate::compatibility::Compatibility;
use crate::description::SkillDescription;
use crate::error::ParseError;
use crate::frontmatter::Frontmatter;
use crate::metadata::Metadata;
use crate::name::SkillName;

/// A complete Agent Skill, combining frontmatter and markdown body.
///
/// This is the main type for working with skills. It can be loaded from
/// a SKILL.md file or constructed programmatically.
///
/// # Examples
///
/// ```
/// use agent_skills::Skill;
///
/// // Parse from SKILL.md content
/// let content = r#"---
/// name: my-skill
/// description: Does something useful.
/// ---
/// # Instructions
///
/// Follow these steps...
/// "#;
///
/// let skill = Skill::parse(content).unwrap();
/// assert_eq!(skill.name().as_str(), "my-skill");
/// assert!(skill.body().contains("# Instructions"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    frontmatter: Frontmatter,
    body: String,
}

impl Skill {
    /// Creates a new skill with frontmatter and body.
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_skills::{Skill, Frontmatter, SkillName, SkillDescription};
    ///
    /// let name = SkillName::new("my-skill").unwrap();
    /// let desc = SkillDescription::new("Does something.").unwrap();
    /// let frontmatter = Frontmatter::new(name, desc);
    ///
    /// let skill = Skill::new(frontmatter, "# Instructions\n\nDo this.");
    /// ```
    #[must_use]
    pub fn new(frontmatter: Frontmatter, body: impl Into<String>) -> Self {
        Self {
            frontmatter,
            body: body.into(),
        }
    }

    /// Parses a skill from SKILL.md content.
    ///
    /// The content must start with `---` (the YAML frontmatter delimiter),
    /// followed by valid YAML, another `---` delimiter, and then the
    /// markdown body.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - The content doesn't start with `---`
    /// - The YAML frontmatter is invalid
    /// - Required fields are missing or invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_skills::Skill;
    ///
    /// let content = r#"---
    /// name: my-skill
    /// description: Does something useful.
    /// ---
    /// # Instructions
    ///
    /// Follow these steps...
    /// "#;
    ///
    /// let skill = Skill::parse(content).unwrap();
    /// assert_eq!(skill.name().as_str(), "my-skill");
    /// ```
    pub fn parse(content: &str) -> Result<Self, ParseError> {
        let (yaml, body) = split_frontmatter_and_body(content)?;
        let frontmatter = parse_frontmatter(yaml)?;
        Ok(Self {
            frontmatter,
            body: body.to_string(),
        })
    }

    /// Returns the skill name.
    #[must_use]
    pub const fn name(&self) -> &SkillName {
        self.frontmatter.name()
    }

    /// Returns the skill description.
    #[must_use]
    pub const fn description(&self) -> &SkillDescription {
        self.frontmatter.description()
    }

    /// Returns the frontmatter.
    #[must_use]
    pub const fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    /// Returns the markdown body (instructions).
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the body trimmed of leading/trailing whitespace.
    #[must_use]
    pub fn body_trimmed(&self) -> &str {
        self.body.trim()
    }
}

/// Splits SKILL.md content into frontmatter YAML and body.
fn split_frontmatter_and_body(content: &str) -> Result<(&str, &str), ParseError> {
    // Content must start with ---
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(ParseError::MissingFrontmatter);
    }

    // Find the closing ---
    let after_opening = &content[3..];
    let after_opening = after_opening.trim_start_matches(['\r', '\n']);

    // Look for closing delimiter (must be at start of a line)
    let closing_pos = find_closing_delimiter(after_opening);

    closing_pos.map_or(Err(ParseError::UnterminatedFrontmatter), |pos| {
        let yaml = &after_opening[..pos];
        let body = &after_opening[pos + 3..];
        // Strip leading newline from body if present
        let body = body.strip_prefix("\r\n").unwrap_or(body);
        let body = body.strip_prefix('\n').unwrap_or(body);
        Ok((yaml.trim(), body))
    })
}

/// Finds the position of the closing `---` delimiter.
fn find_closing_delimiter(content: &str) -> Option<usize> {
    let mut pos = 0;
    for line in content.lines() {
        if line == "---" {
            return Some(pos);
        }
        // +1 for the newline (this is approximate but works for finding start)
        pos += line.len() + 1;
    }
    None
}

/// Internal struct for YAML deserialization.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawFrontmatter {
    name: Option<String>,
    description: Option<String>,
    license: Option<String>,
    compatibility: Option<String>,
    metadata: Option<HashMap<String, String>>,
    allowed_tools: Option<String>,
}

/// Parses frontmatter from YAML content.
fn parse_frontmatter(yaml: &str) -> Result<Frontmatter, ParseError> {
    let raw: RawFrontmatter = serde_yaml::from_str(yaml).map_err(|e| ParseError::InvalidYaml {
        message: e.to_string(),
    })?;

    // Validate required fields
    let name_str = raw.name.ok_or(ParseError::MissingField { field: "name" })?;
    let desc_str = raw.description.ok_or(ParseError::MissingField {
        field: "description",
    })?;

    // Validate and create types
    let name = SkillName::new(name_str).map_err(ParseError::InvalidName)?;
    let description = SkillDescription::new(desc_str).map_err(ParseError::InvalidDescription)?;

    let compatibility = raw
        .compatibility
        .map(Compatibility::new)
        .transpose()
        .map_err(ParseError::InvalidCompatibility)?;

    let metadata = raw.metadata.map(Metadata::from_pairs);
    let allowed_tools = raw.allowed_tools.map(|s| AllowedTools::new(&s));

    // Build frontmatter, conditionally adding optional fields
    let mut builder = Frontmatter::builder(name, description);

    if let Some(license) = raw.license {
        builder = builder.license(license);
    }

    if let Some(compat) = compatibility {
        builder = builder.compatibility(compat);
    }

    if let Some(meta) = metadata {
        builder = builder.metadata(meta);
    }

    if let Some(tools) = allowed_tools {
        builder = builder.allowed_tools(tools);
    }

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_skill() {
        let content = r#"---
name: my-skill
description: Does something useful.
---
# Instructions

Follow these steps.
"#;
        let skill = Skill::parse(content);
        assert!(skill.is_ok(), "Expected Ok, got: {:?}", skill);
        let skill = skill.unwrap();
        assert_eq!(skill.name().as_str(), "my-skill");
        assert_eq!(skill.description().as_str(), "Does something useful.");
        assert!(skill.body().contains("# Instructions"));
    }

    #[test]
    fn parses_skill_with_all_fields() {
        let content = r#"---
name: pdf-processing
description: Extracts text and tables from PDF files.
license: Apache-2.0
compatibility: Requires poppler-utils
metadata:
  author: example-org
  version: "1.0"
allowed-tools: Bash(git:*) Read Write
---
# PDF Processing

Instructions here.
"#;
        let skill = Skill::parse(content).unwrap();
        assert_eq!(skill.name().as_str(), "pdf-processing");
        assert_eq!(skill.frontmatter().license(), Some("Apache-2.0"));
        assert!(skill.frontmatter().compatibility().is_some());
        assert!(skill.frontmatter().metadata().is_some());
        let metadata = skill.frontmatter().metadata().unwrap();
        assert_eq!(metadata.get("author"), Some("example-org"));
        assert!(skill.frontmatter().allowed_tools().is_some());
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let content = "# No frontmatter here";
        let result = Skill::parse(content);
        assert_eq!(result, Err(ParseError::MissingFrontmatter));
    }

    #[test]
    fn rejects_unterminated_frontmatter() {
        let content = r#"---
name: my-skill
description: Test.
"#;
        let result = Skill::parse(content);
        assert_eq!(result, Err(ParseError::UnterminatedFrontmatter));
    }

    #[test]
    fn rejects_missing_name() {
        let content = r#"---
description: Test.
---
Body
"#;
        let result = Skill::parse(content);
        assert!(matches!(
            result,
            Err(ParseError::MissingField { field: "name" })
        ));
    }

    #[test]
    fn rejects_missing_description() {
        let content = r#"---
name: my-skill
---
Body
"#;
        let result = Skill::parse(content);
        assert!(matches!(
            result,
            Err(ParseError::MissingField {
                field: "description"
            })
        ));
    }

    #[test]
    fn rejects_invalid_name() {
        let content = r#"---
name: Invalid-Name
description: Test.
---
Body
"#;
        let result = Skill::parse(content);
        assert!(matches!(result, Err(ParseError::InvalidName(_))));
    }

    #[test]
    fn rejects_empty_description() {
        let content = r#"---
name: my-skill
description: ""
---
Body
"#;
        let result = Skill::parse(content);
        assert!(matches!(result, Err(ParseError::InvalidDescription(_))));
    }

    #[test]
    fn rejects_invalid_yaml() {
        let content = r#"---
name: my-skill
description [invalid yaml
---
Body
"#;
        let result = Skill::parse(content);
        assert!(matches!(result, Err(ParseError::InvalidYaml { .. })));
    }

    #[test]
    fn body_trimmed_removes_whitespace() {
        let content = r#"---
name: my-skill
description: Test.
---

  Content here

"#;
        let skill = Skill::parse(content).unwrap();
        assert_eq!(skill.body_trimmed(), "Content here");
    }

    #[test]
    fn handles_empty_body() {
        let content = r#"---
name: my-skill
description: Test.
---
"#;
        let skill = Skill::parse(content).unwrap();
        assert!(skill.body().is_empty() || skill.body().trim().is_empty());
    }

    #[test]
    fn handles_leading_whitespace_before_frontmatter() {
        let content = r#"
---
name: my-skill
description: Test.
---
Body
"#;
        let skill = Skill::parse(content);
        assert!(skill.is_ok());
    }

    #[test]
    fn new_creates_skill_directly() {
        let name = SkillName::new("my-skill").unwrap();
        let desc = SkillDescription::new("Test description.").unwrap();
        let frontmatter = Frontmatter::new(name, desc);
        let skill = Skill::new(frontmatter, "# Body content");

        assert_eq!(skill.name().as_str(), "my-skill");
        assert_eq!(skill.body(), "# Body content");
    }
}
