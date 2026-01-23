//! Directory-based skill loading.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::LoadError;
use crate::skill::Skill;

/// A loaded skill directory containing the skill and its files.
///
/// Provides access to the parsed SKILL.md as well as optional directories
/// (scripts/, references/, assets/).
///
/// # Examples
///
/// ```no_run
/// use agent_skills::SkillDirectory;
/// use std::path::Path;
///
/// let dir = SkillDirectory::load(Path::new("./my-skill")).unwrap();
/// println!("Loaded skill: {}", dir.skill().name());
///
/// // Check for optional directories
/// if dir.has_scripts() {
///     for script in dir.scripts().unwrap() {
///         println!("Found script: {}", script.display());
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SkillDirectory {
    path: PathBuf,
    skill: Skill,
}

impl SkillDirectory {
    /// Loads a skill from a directory path.
    ///
    /// The directory must contain a SKILL.md file. The skill name in the
    /// frontmatter must match the directory name (per the specification).
    ///
    /// # Errors
    ///
    /// Returns `LoadError` if:
    /// - The directory doesn't exist
    /// - SKILL.md is missing
    /// - SKILL.md cannot be read
    /// - SKILL.md cannot be parsed
    /// - The skill name doesn't match the directory name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use agent_skills::SkillDirectory;
    /// use std::path::Path;
    ///
    /// let dir = SkillDirectory::load(Path::new("./my-skill")).unwrap();
    /// assert_eq!(dir.skill().name().as_str(), "my-skill");
    /// ```
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let path = path.as_ref();
        let path_str = path.display().to_string();

        // Check directory exists
        if !path.exists() {
            return Err(LoadError::DirectoryNotFound { path: path_str });
        }

        if !path.is_dir() {
            return Err(LoadError::DirectoryNotFound { path: path_str });
        }

        // Read SKILL.md
        let skill_file = path.join("SKILL.md");
        if !skill_file.exists() {
            return Err(LoadError::SkillFileNotFound { path: path_str });
        }

        let content = fs::read_to_string(&skill_file).map_err(|e| LoadError::IoError {
            path: skill_file.display().to_string(),
            kind: e.kind(),
            message: e.to_string(),
        })?;

        // Parse skill
        let skill = Skill::parse(&content)?;

        // Validate name matches directory name
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if skill.name().as_str() != dir_name {
            return Err(LoadError::NameMismatch {
                directory_name: dir_name.to_string(),
                skill_name: skill.name().as_str().to_string(),
            });
        }

        Ok(Self {
            path: path.to_path_buf(),
            skill,
        })
    }

    /// Returns the directory path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the parsed skill.
    #[must_use]
    pub const fn skill(&self) -> &Skill {
        &self.skill
    }

    /// Returns `true` if a scripts/ directory exists.
    #[must_use]
    pub fn has_scripts(&self) -> bool {
        self.path.join("scripts").is_dir()
    }

    /// Returns `true` if a references/ directory exists.
    #[must_use]
    pub fn has_references(&self) -> bool {
        self.path.join("references").is_dir()
    }

    /// Returns `true` if an assets/ directory exists.
    #[must_use]
    pub fn has_assets(&self) -> bool {
        self.path.join("assets").is_dir()
    }

    /// Lists files in the scripts/ directory.
    ///
    /// Returns an empty vector if the scripts/ directory doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns `LoadError::IoError` if the directory cannot be read.
    pub fn scripts(&self) -> Result<Vec<PathBuf>, LoadError> {
        list_files_in_subdir(&self.path, "scripts")
    }

    /// Lists files in the references/ directory.
    ///
    /// Returns an empty vector if the references/ directory doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns `LoadError::IoError` if the directory cannot be read.
    pub fn references(&self) -> Result<Vec<PathBuf>, LoadError> {
        list_files_in_subdir(&self.path, "references")
    }

    /// Lists files in the assets/ directory.
    ///
    /// Returns an empty vector if the assets/ directory doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns `LoadError::IoError` if the directory cannot be read.
    pub fn assets(&self) -> Result<Vec<PathBuf>, LoadError> {
        list_files_in_subdir(&self.path, "assets")
    }

    /// Reads a reference file by name.
    ///
    /// The name is relative to the references/ directory.
    ///
    /// # Errors
    ///
    /// Returns `LoadError::FileNotFound` if the file doesn't exist.
    /// Returns `LoadError::IoError` if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use agent_skills::SkillDirectory;
    /// use std::path::Path;
    ///
    /// let dir = SkillDirectory::load(Path::new("./my-skill")).unwrap();
    /// let content = dir.read_reference("REFERENCE.md").unwrap();
    /// ```
    pub fn read_reference(&self, name: &str) -> Result<String, LoadError> {
        let file_path = self.path.join("references").join(name);
        read_file_as_string(&file_path)
    }

    /// Reads a script file by name.
    ///
    /// The name is relative to the scripts/ directory.
    ///
    /// # Errors
    ///
    /// Returns `LoadError::FileNotFound` if the file doesn't exist.
    /// Returns `LoadError::IoError` if the file cannot be read.
    pub fn read_script(&self, name: &str) -> Result<String, LoadError> {
        let file_path = self.path.join("scripts").join(name);
        read_file_as_string(&file_path)
    }

    /// Reads an asset file by name as bytes.
    ///
    /// The name is relative to the assets/ directory.
    ///
    /// # Errors
    ///
    /// Returns `LoadError::FileNotFound` if the file doesn't exist.
    /// Returns `LoadError::IoError` if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use agent_skills::SkillDirectory;
    /// use std::path::Path;
    ///
    /// let dir = SkillDirectory::load(Path::new("./my-skill")).unwrap();
    /// let bytes = dir.read_asset("template.txt").unwrap();
    /// ```
    pub fn read_asset(&self, name: &str) -> Result<Vec<u8>, LoadError> {
        let file_path = self.path.join("assets").join(name);
        read_file_as_bytes(&file_path)
    }

    /// Reads an asset file by name as a string (UTF-8).
    ///
    /// # Errors
    ///
    /// Returns `LoadError::FileNotFound` if the file doesn't exist.
    /// Returns `LoadError::IoError` if the file cannot be read or is not valid UTF-8.
    pub fn read_asset_string(&self, name: &str) -> Result<String, LoadError> {
        let file_path = self.path.join("assets").join(name);
        read_file_as_string(&file_path)
    }
}

/// Lists files in a subdirectory of a skill directory.
fn list_files_in_subdir(base_path: &Path, subdir: &str) -> Result<Vec<PathBuf>, LoadError> {
    let dir_path = base_path.join(subdir);

    if !dir_path.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&dir_path).map_err(|e| LoadError::IoError {
        path: dir_path.display().to_string(),
        kind: e.kind(),
        message: e.to_string(),
    })?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| LoadError::IoError {
            path: dir_path.display().to_string(),
            kind: e.kind(),
            message: e.to_string(),
        })?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    // Sort for consistent ordering
    files.sort();
    Ok(files)
}

/// Reads a file as a string.
fn read_file_as_string(path: &Path) -> Result<String, LoadError> {
    let path_str = path.display().to_string();

    if !path.exists() {
        return Err(LoadError::FileNotFound { path: path_str });
    }

    fs::read_to_string(path).map_err(|e| LoadError::IoError {
        path: path_str,
        kind: e.kind(),
        message: e.to_string(),
    })
}

/// Reads a file as bytes.
fn read_file_as_bytes(path: &Path) -> Result<Vec<u8>, LoadError> {
    let path_str = path.display().to_string();

    if !path.exists() {
        return Err(LoadError::FileNotFound { path: path_str });
    }

    fs::read(path).map_err(|e| LoadError::IoError {
        path: path_str,
        kind: e.kind(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_skill_dir(temp: &TempDir, name: &str, content: &str) -> PathBuf {
        let skill_dir = temp.path().join(name);
        fs::create_dir(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
        skill_dir
    }

    fn minimal_skill_content(name: &str) -> String {
        format!(
            r#"---
name: {name}
description: Test skill.
---
# Instructions
"#
        )
    }

    #[test]
    fn loads_valid_skill_directory() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));

        let result = SkillDirectory::load(&skill_dir);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let dir = result.unwrap();
        assert_eq!(dir.skill().name().as_str(), "my-skill");
    }

    #[test]
    fn rejects_nonexistent_directory() {
        let result = SkillDirectory::load("/nonexistent/path");
        assert!(matches!(result, Err(LoadError::DirectoryNotFound { .. })));
    }

    #[test]
    fn rejects_missing_skill_file() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("empty-skill");
        fs::create_dir(&skill_dir).unwrap();

        let result = SkillDirectory::load(&skill_dir);
        assert!(matches!(result, Err(LoadError::SkillFileNotFound { .. })));
    }

    #[test]
    fn detects_name_mismatch() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("other-name"));

        let result = SkillDirectory::load(&skill_dir);
        assert!(matches!(
            result,
            Err(LoadError::NameMismatch {
                directory_name,
                skill_name,
            }) if directory_name == "my-skill" && skill_name == "other-name"
        ));
    }

    #[test]
    fn has_scripts_returns_false_without_scripts_dir() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        assert!(!dir.has_scripts());
    }

    #[test]
    fn has_scripts_returns_true_with_scripts_dir() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));
        fs::create_dir(skill_dir.join("scripts")).unwrap();

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        assert!(dir.has_scripts());
    }

    #[test]
    fn lists_scripts() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir(&scripts_dir).unwrap();
        fs::write(scripts_dir.join("run.sh"), "#!/bin/bash").unwrap();
        fs::write(scripts_dir.join("build.py"), "# Python").unwrap();

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        let scripts = dir.scripts().unwrap();
        assert_eq!(scripts.len(), 2);
    }

    #[test]
    fn scripts_returns_empty_without_scripts_dir() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        let scripts = dir.scripts().unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn has_references_works() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        assert!(!dir.has_references());

        fs::create_dir(skill_dir.join("references")).unwrap();
        let dir = SkillDirectory::load(&skill_dir).unwrap();
        assert!(dir.has_references());
    }

    #[test]
    fn has_assets_works() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        assert!(!dir.has_assets());

        fs::create_dir(skill_dir.join("assets")).unwrap();
        let dir = SkillDirectory::load(&skill_dir).unwrap();
        assert!(dir.has_assets());
    }

    #[test]
    fn reads_reference_file() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));
        let refs_dir = skill_dir.join("references");
        fs::create_dir(&refs_dir).unwrap();
        fs::write(refs_dir.join("REFERENCE.md"), "# Reference\n\nContent").unwrap();

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        let content = dir.read_reference("REFERENCE.md").unwrap();
        assert!(content.contains("# Reference"));
    }

    #[test]
    fn read_reference_returns_error_for_missing_file() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        let result = dir.read_reference("nonexistent.md");
        assert!(matches!(result, Err(LoadError::FileNotFound { .. })));
    }

    #[test]
    fn reads_asset_as_bytes() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));
        let assets_dir = skill_dir.join("assets");
        fs::create_dir(&assets_dir).unwrap();
        fs::write(assets_dir.join("data.bin"), &[0x00, 0x01, 0x02]).unwrap();

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        let bytes = dir.read_asset("data.bin").unwrap();
        assert_eq!(bytes, vec![0x00, 0x01, 0x02]);
    }

    #[test]
    fn reads_asset_as_string() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));
        let assets_dir = skill_dir.join("assets");
        fs::create_dir(&assets_dir).unwrap();
        fs::write(assets_dir.join("template.txt"), "Hello, World!").unwrap();

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        let content = dir.read_asset_string("template.txt").unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn reads_script_file() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir(&scripts_dir).unwrap();
        fs::write(scripts_dir.join("run.sh"), "#!/bin/bash\necho hello").unwrap();

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        let content = dir.read_script("run.sh").unwrap();
        assert!(content.contains("#!/bin/bash"));
    }

    #[test]
    fn path_returns_directory_path() {
        let temp = TempDir::new().unwrap();
        let skill_dir = create_skill_dir(&temp, "my-skill", &minimal_skill_content("my-skill"));

        let dir = SkillDirectory::load(&skill_dir).unwrap();
        assert_eq!(dir.path(), skill_dir);
    }
}
