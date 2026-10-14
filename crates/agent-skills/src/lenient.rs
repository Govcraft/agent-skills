//! Opt-in client loading with explicit diagnostics.

use std::path::{Path, PathBuf};

use crate::loader::read_skill_content;
use crate::name::validate_skill_name_syntax;
use crate::skill::{parse_raw_frontmatter, split_frontmatter_and_body};
use crate::{
    AllowedTools, Compatibility, LoadError, Metadata, ParseError, SkillDescription, SkillName,
};

/// A nonfatal deviation accepted by the lenient directory loader.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadDiagnostic {
    /// The name exceeds the specification's Unicode scalar value limit.
    NameTooLong {
        /// Actual number of Unicode scalar values.
        length: usize,
        /// Maximum permitted by strict validation.
        max: usize,
    },
    /// The exact skill name differs from the directory name.
    NameMismatch {
        /// Directory's final component.
        directory_name: String,
        /// Name declared in frontmatter.
        skill_name: String,
    },
}

/// A client-loaded skill that may have an overlong name.
///
/// Names retain strict character and hyphen syntax, but do not enforce the
/// length limit of [`SkillName`]. Input is preserved without normalization.
/// Descriptions and optional fields retain their usual validation rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LenientSkill {
    name: String,
    description: SkillDescription,
    license: Option<String>,
    compatibility: Option<Compatibility>,
    metadata: Option<Metadata>,
    allowed_tools: Option<AllowedTools>,
    body: String,
}

impl LenientSkill {
    fn parse(content: &str) -> Result<Self, ParseError> {
        let (yaml, body) = split_frontmatter_and_body(content)?;
        let raw = parse_raw_frontmatter(yaml)?;
        let name = raw.name.ok_or(ParseError::MissingField { field: "name" })?;
        validate_skill_name_syntax(&name).map_err(ParseError::InvalidName)?;
        let description = raw.description.ok_or(ParseError::MissingField {
            field: "description",
        })?;
        let description =
            SkillDescription::new(description).map_err(ParseError::InvalidDescription)?;
        let compatibility = raw
            .compatibility
            .map(Compatibility::new)
            .transpose()
            .map_err(ParseError::InvalidCompatibility)?;
        Ok(Self {
            name,
            description,
            license: raw.license,
            compatibility,
            metadata: raw.metadata.map(Metadata::from_pairs),
            allowed_tools: raw.allowed_tools.map(|tools| AllowedTools::new(&tools)),
            body: body.to_string(),
        })
    }

    /// Returns the exact name, which may exceed 64 Unicode scalar values.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the validated description.
    #[must_use]
    pub const fn description(&self) -> &SkillDescription {
        &self.description
    }

    /// Returns the license, if present.
    #[must_use]
    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    /// Returns the validated compatibility requirements, if present.
    #[must_use]
    pub const fn compatibility(&self) -> Option<&Compatibility> {
        self.compatibility.as_ref()
    }

    /// Returns metadata, if present.
    #[must_use]
    pub const fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Returns allowed tools, if present.
    #[must_use]
    pub const fn allowed_tools(&self) -> Option<&AllowedTools> {
        self.allowed_tools.as_ref()
    }

    /// Returns the markdown instructions.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

/// A skill directory loaded with nonfatal conformance diagnostics.
#[derive(Debug, Clone)]
pub struct LenientSkillDirectory {
    path: PathBuf,
    skill: LenientSkill,
    diagnostics: Vec<LoadDiagnostic>,
}

impl LenientSkillDirectory {
    /// Loads a directory, relaxing only name length and directory-name matching.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] for filesystem errors, malformed or unknown fields,
    /// missing required fields, invalid name syntax, or invalid field values.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let path = path.as_ref();
        let content = read_skill_content(path)?;
        let skill = LenientSkill::parse(&content)?;
        let mut diagnostics = Vec::new();
        let length = skill.name().chars().count();
        if length > SkillName::MAX_LENGTH {
            diagnostics.push(LoadDiagnostic::NameTooLong {
                length,
                max: SkillName::MAX_LENGTH,
            });
        }
        let directory_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if directory_name != skill.name() {
            diagnostics.push(LoadDiagnostic::NameMismatch {
                directory_name: directory_name.to_string(),
                skill_name: skill.name().to_string(),
            });
        }
        Ok(Self {
            path: path.to_path_buf(),
            skill,
            diagnostics,
        })
    }

    /// Returns the directory path for resolving supporting files.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the client-loaded skill.
    #[must_use]
    pub const fn skill(&self) -> &LenientSkill {
        &self.skill
    }

    /// Returns all nonfatal diagnostics in validation order.
    #[must_use]
    pub fn diagnostics(&self) -> &[LoadDiagnostic] {
        &self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SkillDirectory;
    use std::fs;
    use tempfile::TempDir;

    fn write_skill(temp: &TempDir, name: &str) -> std::io::Result<PathBuf> {
        let path = temp.path().join("skill");
        fs::create_dir_all(&path)?;
        fs::write(
            path.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: Test.\n---\nBody\n"),
        )?;
        Ok(path)
    }

    #[test]
    fn conforming_skill_has_no_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let path = write_skill(&temp, "skill")?;
        let loaded = SkillDirectory::load_lenient(&path)?;
        assert!(loaded.diagnostics().is_empty());
        assert_eq!(loaded.path(), path);
        assert_eq!(
            loaded.skill().name(),
            SkillDirectory::load(&path)?.skill().name().as_str()
        );
        assert_eq!(loaded.skill().body(), "Body\n");
        Ok(())
    }

    #[test]
    fn mismatch_warns_only_in_lenient_loading() -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let path = write_skill(&temp, "other")?;
        assert!(matches!(
            SkillDirectory::load(&path),
            Err(LoadError::NameMismatch { .. })
        ));
        assert_eq!(
            SkillDirectory::load_lenient(path)?.diagnostics(),
            &[LoadDiagnostic::NameMismatch {
                directory_name: "skill".to_string(),
                skill_name: "other".to_string(),
            }]
        );
        Ok(())
    }

    #[test]
    fn unicode_length_warning_uses_scalars_and_preserves_name()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        for length in [64, 65] {
            let name = "é".repeat(length);
            let path = write_skill(&temp, &name)?;
            let loaded = SkillDirectory::load_lenient(&path)?;
            assert_eq!(loaded.skill().name(), name);
            assert_eq!(loaded.diagnostics().len(), if length == 64 { 1 } else { 2 });
            if length == 65 {
                assert_eq!(
                    loaded.diagnostics()[0],
                    LoadDiagnostic::NameTooLong {
                        length: 65,
                        max: 64
                    }
                );
                assert!(matches!(
                    SkillDirectory::load(path),
                    Err(LoadError::ParseError(ParseError::InvalidName(_)))
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn invalid_fields_remain_fatal() -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let path = write_skill(&temp, "skill")?;
        for yaml in [
            "name: skill",
            "description: Test.",
            "name: ''\ndescription: Test.",
            "name: skill\ndescription: '   '",
            "name: skill\ndescription: [bad",
            "name: skill\ndescription: Test.\nextra: typo",
            "name: skill\ndescription: Test.\ncompatibility: ''",
            "name: skill\ndescription: Test.\nmetadata: [bad]",
        ] {
            fs::write(path.join("SKILL.md"), format!("---\n{yaml}\n---\nBody"))?;
            assert!(
                SkillDirectory::load_lenient(&path).is_err(),
                "accepted {yaml}"
            );
        }
        for name in [
            format!("{}X", "a".repeat(65)),
            format!("{}--b", "a".repeat(65)),
            "-bad".to_string(),
            "bad-".to_string(),
        ] {
            write_skill(&temp, &name)?;
            assert!(
                SkillDirectory::load_lenient(&path).is_err(),
                "accepted {name}"
            );
        }
        Ok(())
    }

    #[test]
    fn optional_fields_are_preserved() -> Result<(), Box<dyn std::error::Error>> {
        let temp = TempDir::new()?;
        let path = write_skill(&temp, "skill")?;
        fs::write(
            path.join("SKILL.md"),
            "---\nname: skill\ndescription: Test.\nlicense: MIT\ncompatibility: Linux\nmetadata:\n  author: tester\nallowed-tools: Read Write\n---\nBody",
        )?;
        let loaded = SkillDirectory::load_lenient(path)?;
        let skill = loaded.skill();
        assert_eq!(skill.description().as_str(), "Test.");
        assert_eq!(skill.license(), Some("MIT"));
        assert_eq!(
            skill.compatibility().map(Compatibility::as_str),
            Some("Linux")
        );
        assert_eq!(
            skill.metadata().and_then(|metadata| metadata.get("author")),
            Some("tester")
        );
        assert!(skill.allowed_tools().is_some());
        Ok(())
    }
}
