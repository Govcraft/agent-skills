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
/// let content = r"---
/// name: my-skill
/// description: Does something useful.
/// ---
/// ## Instructions
///
/// Follow these steps...
/// ";
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
    /// - Unknown top-level frontmatter fields are present
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_skills::Skill;
    ///
    /// let content = r"---
    /// name: my-skill
    /// description: Does something useful.
    /// ---
    /// ## Instructions
    ///
    /// Follow these steps...
    /// ";
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
pub fn split_frontmatter_and_body(content: &str) -> Result<(&str, &str), ParseError> {
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
    for line in content.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some(pos);
        }
        // Preserve the actual newline width when calculating byte offsets.
        pos += line.len();
    }
    None
}

/// Internal struct for YAML deserialization.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub allowed_tools: Option<String>,
}

/// Deserializes known frontmatter fields without validating domain values.
pub fn parse_raw_frontmatter(yaml: &str) -> Result<RawFrontmatter, ParseError> {
    serde_yml::from_str(yaml).map_err(|error| ParseError::InvalidYaml {
        message: error.to_string(),
    })
}

/// Parses frontmatter from YAML content.
fn parse_frontmatter(yaml: &str) -> Result<Frontmatter, ParseError> {
    let raw = parse_raw_frontmatter(yaml)?;

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
    fn parses_crlf_frontmatter_without_truncating_fields_or_body() -> Result<(), ParseError> {
        let content = "---\r\nname: test-skill\r\ndescription: Test.\r\nlicense: MIT\r\ncompatibility: Requires git\r\nmetadata:\r\n  author: example\r\nallowed-tools: Read Write\r\n---\r\n# Instructions\r\nKeep this body.\r\n";
        let skill = Skill::parse(content)?;
        assert_eq!(skill.frontmatter().license(), Some("MIT"));
        assert_eq!(
            skill
                .frontmatter()
                .compatibility()
                .map(Compatibility::as_str),
            Some("Requires git")
        );
        assert_eq!(
            skill
                .frontmatter()
                .metadata()
                .and_then(|metadata| metadata.get("author")),
            Some("example")
        );
        assert_eq!(
            skill.frontmatter().allowed_tools().map(ToString::to_string),
            Some("Read Write".to_string())
        );
        assert_eq!(skill.body(), "# Instructions\r\nKeep this body.\r\n");
        Ok(())
    }

    #[test]
    fn rejects_unknown_frontmatter_fields_with_actionable_errors() {
        for unknown in ["invented-field", "descripton", "allowed_tools"] {
            let content =
                format!("---\nname: test-skill\ndescription: Test.\n{unknown}: value\n---\n");
            let result = Skill::parse(&content);
            assert!(matches!(result, Err(ParseError::InvalidYaml { message })
                if message.contains(unknown) && message.contains("expected") && message.contains("description")));
        }
    }

    #[test]
    fn preserves_arbitrary_metadata_keys() -> Result<(), ParseError> {
        let skill = Skill::parse(
            "---\nname: test-skill\ndescription: Test.\nmetadata:\n  invented-field: value\n  vendor.example/key: custom\n---\n",
        )?;
        let metadata = skill.frontmatter().metadata();
        assert_eq!(
            metadata.and_then(|value| value.get("invented-field")),
            Some("value")
        );
        assert_eq!(
            metadata.and_then(|value| value.get("vendor.example/key")),
            Some("custom")
        );
        Ok(())
    }

    #[test]
    fn parses_minimal_skill() -> Result<(), ParseError> {
        let content = r"---
name: my-skill
description: Does something useful.
---
# Instructions

Follow these steps.
";
        let skill = Skill::parse(content);
        assert!(skill.is_ok(), "Expected Ok, got: {skill:?}");
        let skill = skill?;
        assert_eq!(skill.name().as_str(), "my-skill");
        assert_eq!(skill.description().as_str(), "Does something useful.");
        assert!(skill.body().contains("# Instructions"));
        Ok(())
    }

    #[test]
    fn parses_skill_with_all_fields() -> Result<(), ParseError> {
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
        let skill = Skill::parse(content)?;
        assert_eq!(skill.name().as_str(), "pdf-processing");
        assert_eq!(skill.frontmatter().license(), Some("Apache-2.0"));
        assert!(skill.frontmatter().compatibility().is_some());
        assert!(skill.frontmatter().metadata().is_some());
        assert_eq!(
            skill
                .frontmatter()
                .metadata()
                .and_then(|metadata| metadata.get("author")),
            Some("example-org")
        );
        assert!(skill.frontmatter().allowed_tools().is_some());
        Ok(())
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let content = "# No frontmatter here";
        let result = Skill::parse(content);
        assert_eq!(result, Err(ParseError::MissingFrontmatter));
    }

    #[test]
    fn rejects_unterminated_frontmatter() {
        let content = r"---
name: my-skill
description: Test.
";
        let result = Skill::parse(content);
        assert_eq!(result, Err(ParseError::UnterminatedFrontmatter));
    }

    #[test]
    fn rejects_missing_name() {
        let content = r"---
description: Test.
---
Body
";
        let result = Skill::parse(content);
        assert!(matches!(
            result,
            Err(ParseError::MissingField { field: "name" })
        ));
    }

    #[test]
    fn rejects_missing_description() {
        let content = r"---
name: my-skill
---
Body
";
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
        let content = r"---
name: Invalid-Name
description: Test.
---
Body
";
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
        let content = r"---
name: my-skill
description [invalid yaml
---
Body
";
        let result = Skill::parse(content);
        assert!(matches!(result, Err(ParseError::InvalidYaml { .. })));
    }

    #[test]
    fn body_trimmed_removes_whitespace() -> Result<(), ParseError> {
        let content = r"---
name: my-skill
description: Test.
---

  Content here

";
        let skill = Skill::parse(content)?;
        assert_eq!(skill.body_trimmed(), "Content here");
        Ok(())
    }

    #[test]
    fn handles_empty_body() -> Result<(), ParseError> {
        let content = r"---
name: my-skill
description: Test.
---
";
        let skill = Skill::parse(content)?;
        assert!(skill.body().is_empty() || skill.body().trim().is_empty());
        Ok(())
    }

    #[test]
    fn handles_leading_whitespace_before_frontmatter() {
        let content = r"
---
name: my-skill
description: Test.
---
Body
";
        let skill = Skill::parse(content);
        assert!(skill.is_ok());
    }

    #[test]
    fn new_creates_skill_directly() -> Result<(), ParseError> {
        let name = SkillName::new("my-skill").map_err(ParseError::InvalidName)?;
        let desc =
            SkillDescription::new("Test description.").map_err(ParseError::InvalidDescription)?;
        let frontmatter = Frontmatter::new(name, desc);
        let skill = Skill::new(frontmatter, "# Body content");

        assert_eq!(skill.name().as_str(), "my-skill");
        assert_eq!(skill.body(), "# Body content");
        Ok(())
    }
}
