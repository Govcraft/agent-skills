//! Frontmatter type representing the YAML header of a SKILL.md file.

use serde::{Deserialize, Serialize};

use crate::allowed_tools::AllowedTools;
use crate::compatibility::Compatibility;
use crate::description::SkillDescription;
use crate::metadata::Metadata;
use crate::name::SkillName;

/// The YAML frontmatter of a SKILL.md file.
///
/// Contains both required fields (name, description) and optional fields
/// (license, compatibility, metadata, allowed-tools).
///
/// # Examples
///
/// ```
/// use agent_skills::{Frontmatter, SkillName, SkillDescription};
///
/// let name = SkillName::new("my-skill").unwrap();
/// let description = SkillDescription::new("Does something useful.").unwrap();
///
/// let frontmatter = Frontmatter::new(name, description);
/// assert_eq!(frontmatter.name().as_str(), "my-skill");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Frontmatter {
    name: SkillName,
    description: SkillDescription,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility: Option<Compatibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_tools: Option<AllowedTools>,
}

impl Frontmatter {
    /// Creates a new frontmatter with required fields only.
    ///
    /// Use the builder for setting optional fields.
    #[must_use]
    pub const fn new(name: SkillName, description: SkillDescription) -> Self {
        Self {
            name,
            description,
            license: None,
            compatibility: None,
            metadata: None,
            allowed_tools: None,
        }
    }

    /// Returns a builder for constructing frontmatter with optional fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_skills::{Frontmatter, SkillName, SkillDescription, Metadata};
    ///
    /// let name = SkillName::new("my-skill").unwrap();
    /// let desc = SkillDescription::new("Does something.").unwrap();
    ///
    /// let frontmatter = Frontmatter::builder(name, desc)
    ///     .license("MIT")
    ///     .metadata(Metadata::from_pairs([("author", "test")]))
    ///     .build();
    ///
    /// assert_eq!(frontmatter.license(), Some("MIT"));
    /// ```
    #[must_use]
    pub const fn builder(name: SkillName, description: SkillDescription) -> FrontmatterBuilder {
        FrontmatterBuilder::new(name, description)
    }

    /// Returns the skill name.
    #[must_use]
    pub const fn name(&self) -> &SkillName {
        &self.name
    }

    /// Returns the skill description.
    #[must_use]
    pub const fn description(&self) -> &SkillDescription {
        &self.description
    }

    /// Returns the license, if specified.
    #[must_use]
    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    /// Returns the compatibility string, if specified.
    #[must_use]
    pub const fn compatibility(&self) -> Option<&Compatibility> {
        self.compatibility.as_ref()
    }

    /// Returns the metadata, if specified.
    #[must_use]
    pub const fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Returns the allowed tools, if specified.
    #[must_use]
    pub const fn allowed_tools(&self) -> Option<&AllowedTools> {
        self.allowed_tools.as_ref()
    }
}

/// Builder for constructing [`Frontmatter`] with optional fields.
#[derive(Debug, Clone)]
pub struct FrontmatterBuilder {
    name: SkillName,
    description: SkillDescription,
    license: Option<String>,
    compatibility: Option<Compatibility>,
    metadata: Option<Metadata>,
    allowed_tools: Option<AllowedTools>,
}

impl FrontmatterBuilder {
    /// Creates a new builder with required fields.
    #[must_use]
    pub const fn new(name: SkillName, description: SkillDescription) -> Self {
        Self {
            name,
            description,
            license: None,
            compatibility: None,
            metadata: None,
            allowed_tools: None,
        }
    }

    /// Sets the license.
    #[must_use]
    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Sets the compatibility.
    #[must_use]
    pub fn compatibility(mut self, compat: Compatibility) -> Self {
        self.compatibility = Some(compat);
        self
    }

    /// Sets the metadata.
    #[must_use]
    pub fn metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the allowed tools.
    #[must_use]
    pub fn allowed_tools(mut self, tools: AllowedTools) -> Self {
        self.allowed_tools = Some(tools);
        self
    }

    /// Builds the frontmatter.
    #[must_use]
    pub fn build(self) -> Frontmatter {
        Frontmatter {
            name: self.name,
            description: self.description,
            license: self.license,
            compatibility: self.compatibility,
            metadata: self.metadata,
            allowed_tools: self.allowed_tools,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_name() -> SkillName {
        SkillName::new("test-skill").unwrap()
    }

    fn test_description() -> SkillDescription {
        SkillDescription::new("A test skill.").unwrap()
    }

    #[test]
    fn new_creates_frontmatter_with_required_fields() {
        let fm = Frontmatter::new(test_name(), test_description());
        assert_eq!(fm.name().as_str(), "test-skill");
        assert_eq!(fm.description().as_str(), "A test skill.");
        assert!(fm.license().is_none());
        assert!(fm.compatibility().is_none());
        assert!(fm.metadata().is_none());
        assert!(fm.allowed_tools().is_none());
    }

    #[test]
    fn builder_sets_license() {
        let fm = Frontmatter::builder(test_name(), test_description())
            .license("MIT")
            .build();
        assert_eq!(fm.license(), Some("MIT"));
    }

    #[test]
    fn builder_sets_compatibility() {
        let compat = Compatibility::new("Requires docker").unwrap();
        let fm = Frontmatter::builder(test_name(), test_description())
            .compatibility(compat)
            .build();
        assert!(fm.compatibility().is_some());
        assert_eq!(fm.compatibility().unwrap().as_str(), "Requires docker");
    }

    #[test]
    fn builder_sets_metadata() {
        let metadata = Metadata::from_pairs([("author", "test")]);
        let fm = Frontmatter::builder(test_name(), test_description())
            .metadata(metadata)
            .build();
        assert!(fm.metadata().is_some());
        assert_eq!(fm.metadata().unwrap().get("author"), Some("test"));
    }

    #[test]
    fn builder_sets_allowed_tools() {
        let tools = AllowedTools::new("Read Write");
        let fm = Frontmatter::builder(test_name(), test_description())
            .allowed_tools(tools)
            .build();
        assert!(fm.allowed_tools().is_some());
        assert_eq!(fm.allowed_tools().unwrap().len(), 2);
    }

    #[test]
    fn builder_chains_all_options() {
        let compat = Compatibility::new("Requires git").unwrap();
        let metadata = Metadata::from_pairs([("version", "1.0")]);
        let tools = AllowedTools::new("Bash");

        let fm = Frontmatter::builder(test_name(), test_description())
            .license("Apache-2.0")
            .compatibility(compat)
            .metadata(metadata)
            .allowed_tools(tools)
            .build();

        assert_eq!(fm.license(), Some("Apache-2.0"));
        assert!(fm.compatibility().is_some());
        assert!(fm.metadata().is_some());
        assert!(fm.allowed_tools().is_some());
    }
}
