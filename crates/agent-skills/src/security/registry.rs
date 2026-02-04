//! Secure skill registry with attestation verification.
//!
//! This module provides [`SecureSkillRegistry`], which wraps skill loading
//! with cryptographic verification including agent-uri identity validation,
//! PASETO attestation verification, and capability-based access control.

use std::collections::HashMap;
use std::path::Path;

use agent_uri::AgentUri;
use agent_uri_attestation::{AttestationClaims, Verifier, VerifyingKey};

use crate::security::cache::{AttestationCache, CacheConfig};
use crate::security::capabilities::{max_trust_tier_for_capabilities, validate_trust_tier};
use crate::security::error::SkillSecurityError;
use crate::security::registry_client::{RegistryClient, RegistryClientConfig};
use crate::security::verified::{SkillId, TrustTier, VerifiedSkill};
use crate::{LoadError, Skill, SkillDirectory};

/// Configuration for the secure skill registry.
#[derive(Debug, Clone)]
pub struct SecureSkillRegistryConfig {
    /// Whether to require attestations for all skills.
    pub require_attestation: bool,
    /// Default trust tier for the current session.
    pub default_trust_tier: TrustTier,
    /// Cache configuration.
    pub cache_config: CacheConfig,
    /// Registry client configuration.
    pub registry_config: RegistryClientConfig,
    /// The key name in skill metadata for agent-uri.
    pub agent_uri_key: String,
}

impl Default for SecureSkillRegistryConfig {
    fn default() -> Self {
        Self {
            require_attestation: true,
            default_trust_tier: TrustTier::Standard,
            cache_config: CacheConfig::default(),
            registry_config: RegistryClientConfig::default(),
            agent_uri_key: "agent-uri".to_string(),
        }
    }
}

impl SecureSkillRegistryConfig {
    /// Creates a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether attestations are required.
    #[must_use]
    pub const fn require_attestation(mut self, required: bool) -> Self {
        self.require_attestation = required;
        self
    }

    /// Sets the default trust tier.
    #[must_use]
    pub const fn default_trust_tier(mut self, tier: TrustTier) -> Self {
        self.default_trust_tier = tier;
        self
    }

    /// Sets the cache configuration.
    #[must_use]
    pub const fn cache_config(mut self, config: CacheConfig) -> Self {
        self.cache_config = config;
        self
    }

    /// Sets the registry client configuration.
    #[must_use]
    pub fn registry_config(mut self, config: RegistryClientConfig) -> Self {
        self.registry_config = config;
        self
    }

    /// Sets the metadata key used for agent-uri.
    #[must_use]
    pub fn agent_uri_key(mut self, key: impl Into<String>) -> Self {
        self.agent_uri_key = key.into();
        self
    }
}

/// A secure skill registry with attestation verification.
///
/// This registry wraps skill loading with security verification:
///
/// - Extracts and validates agent-uri from skill frontmatter
/// - Verifies PASETO attestation tokens from trusted roots
/// - Validates capabilities against the session trust tier
/// - Caches verified attestations for performance
///
/// # Example
///
/// ```ignore
/// use agent_skills::security::{
///     SecureSkillRegistry,
///     SecureSkillRegistryConfig,
///     TrustTier,
/// };
/// use agent_uri_attestation::VerifyingKey;
///
/// // Create registry
/// let mut registry = SecureSkillRegistry::new(SecureSkillRegistryConfig::new());
///
/// // Add trusted roots
/// registry.add_trusted_root("acme.com", verifying_key);
///
/// // Load and verify a skill
/// let verified = registry.load_verified("/path/to/skill")?;
///
/// println!("Loaded: {} (trust: {})", verified.name(), verified.trust_tier());
/// ```
#[derive(Debug)]
pub struct SecureSkillRegistry {
    /// Configuration for this registry.
    config: SecureSkillRegistryConfig,
    /// Verifier for attestation tokens.
    verifier: Verifier,
    /// Cache for verified attestations.
    cache: AttestationCache,
    /// HTTP client for fetching attestations.
    registry_client: RegistryClient,
    /// Currently verified skills, indexed by skill name.
    verified_skills: HashMap<String, VerifiedSkill>,
}

impl SecureSkillRegistry {
    /// Creates a new secure skill registry with the given configuration.
    #[must_use]
    pub fn new(config: SecureSkillRegistryConfig) -> Self {
        Self {
            cache: AttestationCache::with_config(config.cache_config.clone()),
            registry_client: RegistryClient::new(config.registry_config.clone()),
            config,
            verifier: Verifier::new(),
            verified_skills: HashMap::new(),
        }
    }

    /// Adds a trusted root for attestation verification.
    ///
    /// # Arguments
    ///
    /// * `trust_root` - The trust root identifier (e.g., "acme.com")
    /// * `public_key` - The Ed25519 public key for this trust root
    pub fn add_trusted_root(&mut self, trust_root: impl Into<String>, public_key: VerifyingKey) {
        self.verifier.add_trusted_root(trust_root, public_key);
    }

    /// Returns true if the given trust root is registered.
    #[must_use]
    pub fn has_trusted_root(&self, trust_root: &str) -> bool {
        self.verifier.has_trusted_root(trust_root)
    }

    /// Returns the number of registered trusted roots.
    #[must_use]
    pub fn trusted_root_count(&self) -> usize {
        self.verifier.trusted_root_count()
    }

    /// Loads and verifies a skill from a directory path.
    ///
    /// This method:
    /// 1. Loads the skill from the directory
    /// 2. Extracts the agent-uri from frontmatter metadata
    /// 3. Verifies the attestation (from cache or registry)
    /// 4. Validates capabilities against the session trust tier
    /// 5. Returns a `VerifiedSkill` if all checks pass
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the skill directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The skill cannot be loaded from the path
    /// - The skill is missing an agent-uri
    /// - Attestation verification fails
    /// - Capabilities exceed the session trust tier
    pub fn load_verified(&mut self, path: &Path) -> Result<VerifiedSkill, SkillLoadError> {
        // Load the skill from the directory
        let skill_dir = SkillDirectory::load(path).map_err(SkillLoadError::Load)?;
        let skill = skill_dir.skill().clone();

        // Extract agent-uri from frontmatter metadata
        let agent_uri_str = self.extract_agent_uri(&skill, path)?;

        // Parse the agent-uri
        let agent_uri = AgentUri::parse(&agent_uri_str).map_err(|e| {
            SkillLoadError::Security(SkillSecurityError::InvalidAgentUri {
                uri: agent_uri_str.clone(),
                reason: e.to_string(),
            })
        })?;

        // Get attestation (from cache or verify token)
        let attestation = self.get_or_verify_attestation(&agent_uri, &agent_uri_str)?;

        // Determine trust tier from capabilities
        let trust_tier = max_trust_tier_for_capabilities(&attestation.capabilities);

        // Validate against session trust tier
        validate_trust_tier(self.config.default_trust_tier, &attestation.capabilities)
            .map_err(SkillLoadError::Security)?;

        // Create verified skill
        let verified = VerifiedSkill::new(skill, agent_uri, attestation, trust_tier);

        // Store in registry
        let name = verified.name().to_string();
        self.verified_skills.insert(name, verified.clone());

        Ok(verified)
    }

    /// Loads a skill without requiring attestation verification.
    ///
    /// This method loads a skill but marks it with `TrustTier::Untrusted`.
    /// Use this for skills that don't have attestations or when operating
    /// in a development/testing mode.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the skill directory
    ///
    /// # Errors
    ///
    /// Returns an error if the skill cannot be loaded from the path.
    pub fn load_unverified(&mut self, path: &Path) -> Result<VerifiedSkill, SkillLoadError> {
        // Load the skill from the directory
        let skill_dir = SkillDirectory::load(path).map_err(SkillLoadError::Load)?;
        let skill = skill_dir.skill().clone();

        // Try to extract agent-uri, but don't fail if missing
        let agent_uri_str = self
            .extract_agent_uri(&skill, path)
            .unwrap_or_else(|_| format!("agent://local/unverified/skill_{}", SkillId::new()));

        // Parse the agent-uri
        let agent_uri = AgentUri::parse(&agent_uri_str).map_err(|e| {
            SkillLoadError::Security(SkillSecurityError::InvalidAgentUri {
                uri: agent_uri_str.clone(),
                reason: e.to_string(),
            })
        })?;

        // Create minimal attestation claims for unverified skill
        let attestation = AttestationClaims::builder()
            .agent_uri(&agent_uri_str)
            .issuer("local")
            .build()
            .map_err(|e| SkillLoadError::Security(e.into()))?;

        // Always untrusted for unverified skills
        let verified =
            VerifiedSkill::new(skill, agent_uri, attestation, TrustTier::Untrusted);

        // Store in registry
        let name = verified.name().to_string();
        self.verified_skills.insert(name, verified.clone());

        Ok(verified)
    }

    /// Verifies an attestation token for the given agent URI.
    ///
    /// This method verifies the token signature, checks expiration,
    /// and validates the issuer against trusted roots.
    ///
    /// # Arguments
    ///
    /// * `token` - The PASETO attestation token
    /// * `agent_uri` - The agent URI to verify against
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    pub fn verify_attestation(
        &self,
        token: &str,
        agent_uri: &AgentUri,
    ) -> Result<AttestationClaims, SkillSecurityError> {
        self.verifier
            .verify_for_uri(token, agent_uri)
            .map_err(SkillSecurityError::from)
    }

    /// Returns a verified skill by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&VerifiedSkill> {
        self.verified_skills.get(name)
    }

    /// Returns all verified skills.
    pub fn all(&self) -> impl Iterator<Item = &VerifiedSkill> {
        self.verified_skills.values()
    }

    /// Returns the number of verified skills in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.verified_skills.len()
    }

    /// Returns true if no skills are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.verified_skills.is_empty()
    }

    /// Removes a skill from the registry.
    pub fn remove(&mut self, name: &str) -> Option<VerifiedSkill> {
        self.verified_skills.remove(name)
    }

    /// Clears all skills from the registry.
    pub fn clear(&mut self) {
        self.verified_skills.clear();
    }

    /// Returns the attestation cache.
    #[must_use]
    pub const fn cache(&self) -> &AttestationCache {
        &self.cache
    }

    /// Returns the registry client.
    #[must_use]
    pub const fn registry_client(&self) -> &RegistryClient {
        &self.registry_client
    }

    /// Extracts the agent-uri from skill frontmatter metadata.
    fn extract_agent_uri(&self, skill: &Skill, path: &Path) -> Result<String, SkillLoadError> {
        let metadata = skill.frontmatter().metadata().ok_or_else(|| {
            SkillLoadError::Security(SkillSecurityError::MissingAgentUri {
                skill_path: path.display().to_string(),
            })
        })?;

        metadata
            .get(&self.config.agent_uri_key)
            .map(String::from)
            .ok_or_else(|| {
                SkillLoadError::Security(SkillSecurityError::MissingAgentUri {
                    skill_path: path.display().to_string(),
                })
            })
    }

    /// Gets attestation from cache or verifies the token.
    fn get_or_verify_attestation(
        &self,
        agent_uri: &AgentUri,
        agent_uri_str: &str,
    ) -> Result<AttestationClaims, SkillLoadError> {
        // Check cache first
        if let Some(cached) = self.cache.get(agent_uri_str) {
            return Ok(cached);
        }

        // If no trusted roots are configured and attestation is required, error
        if self.verifier.trusted_root_count() == 0 && self.config.require_attestation {
            return Err(SkillLoadError::Security(
                SkillSecurityError::MissingConfiguration {
                    config_name: "trusted_roots".to_string(),
                },
            ));
        }

        // In production, we would fetch from registry here
        // For now, return an error indicating attestation not available
        Err(SkillLoadError::Security(
            SkillSecurityError::AttestationFetchFailed {
                agent_uri: agent_uri.to_string(),
                reason: "attestation fetch not implemented - provide token via verify_attestation"
                    .to_string(),
            },
        ))
    }

    /// Manually caches an attestation for the given URI.
    ///
    /// This is useful for testing or when attestations are provided
    /// through an alternative channel.
    pub fn cache_attestation(&self, agent_uri: &str, claims: AttestationClaims) {
        self.cache.insert(agent_uri, claims);
    }
}

impl Default for SecureSkillRegistry {
    fn default() -> Self {
        Self::new(SecureSkillRegistryConfig::default())
    }
}

/// Errors that can occur when loading a skill through the secure registry.
#[derive(Debug)]
pub enum SkillLoadError {
    /// Error loading the skill file/directory.
    Load(LoadError),
    /// Security verification error.
    Security(SkillSecurityError),
}

impl std::fmt::Display for SkillLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(e) => write!(f, "skill load error: {e}"),
            Self::Security(e) => write!(f, "security error: {e}"),
        }
    }
}

impl std::error::Error for SkillLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Load(e) => Some(e),
            Self::Security(e) => Some(e),
        }
    }
}

impl From<LoadError> for SkillLoadError {
    fn from(e: LoadError) -> Self {
        Self::Load(e)
    }
}

impl From<SkillSecurityError> for SkillLoadError {
    fn from(e: SkillSecurityError) -> Self {
        Self::Security(e)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use agent_uri_attestation::SigningKey;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_skill_dir(
        dir: &TempDir,
        name: &str,
        agent_uri: Option<&str>,
    ) -> std::path::PathBuf {
        let skill_dir = dir.path().join(name);
        std::fs::create_dir_all(&skill_dir).unwrap();

        let metadata = agent_uri.map_or(String::new(), |uri| {
            format!(
                r#"metadata:
  agent-uri: "{uri}""#
            )
        });

        let content = format!(
            r#"---
name: {name}
description: Test skill for {name}.
{metadata}
---
# {name}

Instructions for {name}.
"#
        );

        std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();
        skill_dir
    }

    #[test]
    fn config_builder_works() {
        let config = SecureSkillRegistryConfig::new()
            .require_attestation(false)
            .default_trust_tier(TrustTier::Elevated)
            .agent_uri_key("custom-uri");

        assert!(!config.require_attestation);
        assert_eq!(config.default_trust_tier, TrustTier::Elevated);
        assert_eq!(config.agent_uri_key, "custom-uri");
    }

    #[test]
    fn registry_creation_works() {
        let registry = SecureSkillRegistry::new(SecureSkillRegistryConfig::new());
        assert!(registry.is_empty());
        assert_eq!(registry.trusted_root_count(), 0);
    }

    #[test]
    fn add_trusted_root_works() {
        let mut registry = SecureSkillRegistry::new(SecureSkillRegistryConfig::new());
        let key = SigningKey::generate();

        registry.add_trusted_root("acme.com", key.verifying_key());

        assert!(registry.has_trusted_root("acme.com"));
        assert!(!registry.has_trusted_root("other.com"));
        assert_eq!(registry.trusted_root_count(), 1);
    }

    #[test]
    fn load_unverified_works() {
        let dir = TempDir::new().unwrap();
        let skill_path = create_test_skill_dir(&dir, "test-skill", None);

        let config = SecureSkillRegistryConfig::new().require_attestation(false);
        let mut registry = SecureSkillRegistry::new(config);

        let verified = registry.load_unverified(&skill_path).unwrap();

        assert_eq!(verified.name(), "test-skill");
        assert_eq!(verified.trust_tier(), TrustTier::Untrusted);
    }

    #[test]
    fn load_verified_requires_agent_uri() {
        let dir = TempDir::new().unwrap();
        let skill_path = create_test_skill_dir(&dir, "test-skill", None);

        let mut registry = SecureSkillRegistry::new(SecureSkillRegistryConfig::new());

        let result = registry.load_verified(&skill_path);
        assert!(matches!(
            result,
            Err(SkillLoadError::Security(SkillSecurityError::MissingAgentUri { .. }))
        ));
    }

    #[test]
    fn load_verified_parses_agent_uri() {
        let dir = TempDir::new().unwrap();
        let skill_path = create_test_skill_dir(
            &dir,
            "test-skill",
            Some("agent://acme.com/test/skill_01h455vb4pex5vsknk084sn02q"),
        );

        let config = SecureSkillRegistryConfig::new().require_attestation(false);
        let mut registry = SecureSkillRegistry::new(config);

        // Will fail at attestation fetch since we don't have a registry
        let result = registry.load_verified(&skill_path);
        assert!(matches!(
            result,
            Err(SkillLoadError::Security(
                SkillSecurityError::AttestationFetchFailed { .. }
            ))
        ));
    }

    #[test]
    fn cache_attestation_works() {
        let registry = SecureSkillRegistry::new(SecureSkillRegistryConfig::new());

        let claims = AttestationClaims::builder()
            .agent_uri("agent://acme.com/test/skill_abc")
            .issuer("acme.com")
            .ttl(Duration::from_secs(3600))
            .build()
            .unwrap();

        registry.cache_attestation("agent://acme.com/test/skill_abc", claims.clone());

        assert!(registry.cache().contains("agent://acme.com/test/skill_abc"));
    }

    #[test]
    fn verify_attestation_with_trusted_root() {
        let mut registry = SecureSkillRegistry::new(SecureSkillRegistryConfig::new());
        let key = SigningKey::generate();

        registry.add_trusted_root("acme.com", key.verifying_key());

        let issuer =
            agent_uri_attestation::Issuer::new("acme.com", key, Duration::from_secs(3600));
        let uri = AgentUri::parse("agent://acme.com/test/skill_01h455vb4pex5vsknk084sn02q").unwrap();
        let token = issuer.issue(&uri, vec!["test".to_string()]).unwrap();

        let claims = registry.verify_attestation(&token, &uri).unwrap();

        assert_eq!(claims.agent_uri, uri.to_string());
        assert_eq!(claims.iss, "acme.com");
    }

    #[test]
    fn get_and_all_skills() {
        let dir = TempDir::new().unwrap();
        let skill_path = create_test_skill_dir(&dir, "test-skill", None);

        let config = SecureSkillRegistryConfig::new().require_attestation(false);
        let mut registry = SecureSkillRegistry::new(config);

        registry.load_unverified(&skill_path).unwrap();

        assert!(registry.get("test-skill").is_some());
        assert!(registry.get("nonexistent").is_none());
        assert_eq!(registry.all().count(), 1);
    }

    #[test]
    fn remove_skill() {
        let dir = TempDir::new().unwrap();
        let skill_path = create_test_skill_dir(&dir, "test-skill", None);

        let config = SecureSkillRegistryConfig::new().require_attestation(false);
        let mut registry = SecureSkillRegistry::new(config);

        registry.load_unverified(&skill_path).unwrap();
        assert_eq!(registry.len(), 1);

        let removed = registry.remove("test-skill");
        assert!(removed.is_some());
        assert!(registry.is_empty());
    }

    #[test]
    fn skill_load_error_display() {
        let err = SkillLoadError::Security(SkillSecurityError::MissingAgentUri {
            skill_path: "/path/to/skill".to_string(),
        });
        let msg = err.to_string();
        assert!(msg.contains("security error"));
        assert!(msg.contains("agent-uri"));
    }
}
