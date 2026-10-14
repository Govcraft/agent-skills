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
/// (license, compatibility, metadata, allowed-tools). Unknown top-level fields
/// are rejected during deserialization; arbitrary string keys are supported
/// within metadata.
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
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
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
    use crate::error::ParseError;

    fn test_name() -> Result<SkillName, ParseError> {
        SkillName::new("test-skill").map_err(ParseError::InvalidName)
    }

    fn test_description() -> Result<SkillDescription, ParseError> {
        SkillDescription::new("A test skill.").map_err(ParseError::InvalidDescription)
    }

    #[test]
    fn deserialization_rejects_unknown_top_level_fields() {
        for unknown in ["invented-field", "descripton", "allowed_tools"] {
            let yaml = format!("name: test-skill\ndescription: Test.\n{unknown}: value\n");
            let result = serde_yml::from_str::<Frontmatter>(&yaml);
            assert!(matches!(result, Err(error) if error.to_string().contains(unknown)));
        }
    }

    #[test]
    fn deserialization_preserves_arbitrary_metadata() -> Result<(), serde_yml::Error> {
        let frontmatter: Frontmatter = serde_yml::from_str(
            "name: test-skill\ndescription: Test.\nmetadata:\n  invented-field: value\n",
        )?;
        assert_eq!(
            frontmatter
                .metadata()
                .and_then(|metadata| metadata.get("invented-field")),
            Some("value")
        );
        Ok(())
    }

    #[test]
    fn new_creates_frontmatter_with_required_fields() -> Result<(), ParseError> {
        let fm = Frontmatter::new(test_name()?, test_description()?);
        assert_eq!(fm.name().as_str(), "test-skill");
        assert_eq!(fm.description().as_str(), "A test skill.");
        assert!(fm.license().is_none());
        assert!(fm.compatibility().is_none());
        assert!(fm.metadata().is_none());
        assert!(fm.allowed_tools().is_none());
        Ok(())
    }

    #[test]
    fn builder_sets_license() -> Result<(), ParseError> {
        let fm = Frontmatter::builder(test_name()?, test_description()?)
            .license("MIT")
            .build();
        assert_eq!(fm.license(), Some("MIT"));
        Ok(())
    }

    #[test]
    fn builder_sets_compatibility() -> Result<(), ParseError> {
        let compat =
            Compatibility::new("Requires docker").map_err(ParseError::InvalidCompatibility)?;
        let fm = Frontmatter::builder(test_name()?, test_description()?)
            .compatibility(compat)
            .build();
        assert!(fm.compatibility().is_some());
        assert_eq!(
            fm.compatibility().map(Compatibility::as_str),
            Some("Requires docker")
        );
        Ok(())
    }

    #[test]
    fn builder_sets_metadata() -> Result<(), ParseError> {
        let metadata = Metadata::from_pairs([("author", "test")]);
        let fm = Frontmatter::builder(test_name()?, test_description()?)
            .metadata(metadata)
            .build();
        assert!(fm.metadata().is_some());
        assert_eq!(
            fm.metadata().and_then(|metadata| metadata.get("author")),
            Some("test")
        );
        Ok(())
    }

    #[test]
    fn builder_sets_allowed_tools() -> Result<(), ParseError> {
        let tools = AllowedTools::new("Read Write");
        let fm = Frontmatter::builder(test_name()?, test_description()?)
            .allowed_tools(tools)
            .build();
        assert!(fm.allowed_tools().is_some());
        assert_eq!(fm.allowed_tools().map(AllowedTools::len), Some(2));
        Ok(())
    }

    #[test]
    fn builder_chains_all_options() -> Result<(), ParseError> {
        let compat =
            Compatibility::new("Requires git").map_err(ParseError::InvalidCompatibility)?;
        let metadata = Metadata::from_pairs([("version", "1.0")]);
        let tools = AllowedTools::new("Bash");

        let fm = Frontmatter::builder(test_name()?, test_description()?)
            .license("Apache-2.0")
            .compatibility(compat)
            .metadata(metadata)
            .allowed_tools(tools)
            .build();

        assert_eq!(fm.license(), Some("Apache-2.0"));
        assert!(fm.compatibility().is_some());
        assert!(fm.metadata().is_some());
        assert!(fm.allowed_tools().is_some());
        Ok(())
    }
}
