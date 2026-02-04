//! Verified skill types with security metadata.
//!
//! This module provides the [`VerifiedSkill`] type, which wraps a standard
//! [`Skill`] with cryptographic verification metadata including attestation
//! claims, capability paths, and trust tier information.

use std::fmt;

use agent_uri::AgentUri;
use agent_uri_attestation::AttestationClaims;
use chrono::{DateTime, Utc};
use mti::prelude::*;

use crate::Skill;

/// Type-safe identifier for skills using the MTI pattern.
///
/// The prefix "skill" ensures these IDs are clearly distinguishable from
/// other entity types in logs and debugging output.
///
/// # Example
///
/// ```
/// use agent_skills::security::SkillId;
///
/// let id = SkillId::new();
/// println!("Created skill: {}", id);
/// // Output: Created skill: skill_01h455vb4pex5vsknk084sn02q
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SkillId(MagicTypeId);

impl SkillId {
    /// Creates a new unique skill identifier.
    #[must_use]
    pub fn new() -> Self {
        Self("skill".create_type_id::<V7>())
    }

    /// Returns the string representation of this ID.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for SkillId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trust tier levels for capability-based access control.
///
/// Trust tiers determine what capabilities a skill can request and
/// what operations it can perform. Higher tiers require stronger
/// attestation and grant more permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TrustTier {
    /// Minimal trust - read-only operations, no network access.
    #[default]
    Untrusted,
    /// Standard trust - basic operations with user approval.
    Standard,
    /// Elevated trust - additional capabilities with verified attestation.
    Elevated,
    /// Full trust - all capabilities, requires strong attestation.
    Full,
}

impl TrustTier {
    /// Returns the string representation of this trust tier.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::Standard => "standard",
            Self::Elevated => "elevated",
            Self::Full => "full",
        }
    }

    /// Returns true if this tier is sufficient for the required tier.
    ///
    /// A tier is sufficient if it is greater than or equal to the required tier.
    #[must_use]
    pub fn is_sufficient_for(&self, required: Self) -> bool {
        *self >= required
    }
}

/// Error returned when parsing a trust tier from a string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTrustTierError {
    /// The invalid input string.
    pub input: String,
}

impl fmt::Display for ParseTrustTierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid trust tier '{}': expected one of 'untrusted', 'standard', 'elevated', 'full'",
            self.input
        )
    }
}

impl std::error::Error for ParseTrustTierError {}

impl std::str::FromStr for TrustTier {
    type Err = ParseTrustTierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "untrusted" => Ok(Self::Untrusted),
            "standard" => Ok(Self::Standard),
            "elevated" => Ok(Self::Elevated),
            "full" => Ok(Self::Full),
            _ => Err(ParseTrustTierError {
                input: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for TrustTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A skill that has been cryptographically verified.
///
/// This type combines a parsed [`Skill`] with security metadata obtained
/// through attestation verification. It provides proof that:
///
/// - The skill's identity (agent-uri) is valid
/// - The skill's attestation was signed by a trusted root
/// - The attestation has not expired
/// - The skill's capabilities match its trust tier
///
/// # Example
///
/// ```ignore
/// use agent_skills::security::{SecureSkillRegistry, VerifiedSkill};
///
/// let registry = SecureSkillRegistry::new(config)?;
/// let verified = registry.load_verified("/path/to/skill").await?;
///
/// println!("Verified skill: {}", verified.skill().name());
/// println!("Trust tier: {}", verified.trust_tier());
/// println!("Capabilities: {:?}", verified.capabilities());
/// ```
#[derive(Debug, Clone)]
pub struct VerifiedSkill {
    /// The unique identifier for this verified skill instance.
    id: SkillId,

    /// The underlying parsed skill.
    skill: Skill,

    /// The verified agent URI from the skill's frontmatter.
    agent_uri: AgentUri,

    /// The verified attestation claims.
    attestation: AttestationClaims,

    /// The capabilities granted by the attestation.
    capabilities: Vec<String>,

    /// The trust tier for this skill based on its capabilities.
    trust_tier: TrustTier,

    /// When this verification was performed.
    verified_at: DateTime<Utc>,
}

impl VerifiedSkill {
    /// Creates a new verified skill.
    ///
    /// This constructor should only be called by the verification pipeline
    /// after successful attestation validation.
    ///
    /// # Arguments
    ///
    /// * `skill` - The parsed skill
    /// * `agent_uri` - The verified agent URI
    /// * `attestation` - The verified attestation claims
    /// * `trust_tier` - The determined trust tier
    #[must_use]
    pub fn new(
        skill: Skill,
        agent_uri: AgentUri,
        attestation: AttestationClaims,
        trust_tier: TrustTier,
    ) -> Self {
        let capabilities = attestation.capabilities.clone();
        Self {
            id: SkillId::new(),
            skill,
            agent_uri,
            attestation,
            capabilities,
            trust_tier,
            verified_at: Utc::now(),
        }
    }

    /// Returns the unique identifier for this verified skill.
    #[must_use]
    pub const fn id(&self) -> &SkillId {
        &self.id
    }

    /// Returns a reference to the underlying skill.
    #[must_use]
    pub const fn skill(&self) -> &Skill {
        &self.skill
    }

    /// Returns the skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.skill.name().as_str()
    }

    /// Returns a reference to the verified agent URI.
    #[must_use]
    pub const fn agent_uri(&self) -> &AgentUri {
        &self.agent_uri
    }

    /// Returns a reference to the attestation claims.
    #[must_use]
    pub const fn attestation(&self) -> &AttestationClaims {
        &self.attestation
    }

    /// Returns the capabilities granted by the attestation.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// Returns the trust tier for this skill.
    #[must_use]
    pub const fn trust_tier(&self) -> TrustTier {
        self.trust_tier
    }

    /// Returns when this skill was verified.
    #[must_use]
    pub const fn verified_at(&self) -> DateTime<Utc> {
        self.verified_at
    }

    /// Returns true if the attestation has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.attestation.is_expired()
    }

    /// Returns true if this skill's trust tier is sufficient for the required tier.
    #[must_use]
    pub fn has_trust_tier(&self, required: TrustTier) -> bool {
        self.trust_tier.is_sufficient_for(required)
    }

    /// Returns the trust root from the agent URI.
    #[must_use]
    pub fn trust_root(&self) -> &str {
        self.agent_uri.trust_root().as_str()
    }

    /// Returns the issuer from the attestation.
    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.attestation.iss
    }

    /// Returns the skill's markdown instructions.
    #[must_use]
    pub fn instructions(&self) -> &str {
        self.skill.body()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_uri() -> AgentUri {
        AgentUri::parse("agent://acme.com/workflow/approval/skill_01h455vb4pex5vsknk084sn02q")
            .unwrap()
    }

    fn test_claims() -> AttestationClaims {
        AttestationClaims::builder()
            .agent_uri("agent://acme.com/workflow/approval/skill_01h455vb4pex5vsknk084sn02q")
            .issuer("acme.com")
            .add_capability("workflow/approval")
            .ttl(Duration::from_secs(3600))
            .build()
            .unwrap()
    }

    fn test_skill() -> Skill {
        Skill::parse(
            r#"---
name: test-skill
description: A test skill for verification.
---
# Test Skill

Instructions here.
"#,
        )
        .unwrap()
    }

    #[test]
    fn skill_id_generates_unique_ids() {
        let id1 = SkillId::new();
        let id2 = SkillId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn skill_id_has_correct_prefix() {
        let id = SkillId::new();
        assert!(id.to_string().starts_with("skill_"));
    }

    #[test]
    fn trust_tier_ordering_is_correct() {
        assert!(TrustTier::Untrusted < TrustTier::Standard);
        assert!(TrustTier::Standard < TrustTier::Elevated);
        assert!(TrustTier::Elevated < TrustTier::Full);
    }

    #[test]
    fn trust_tier_from_str_is_case_insensitive() {
        assert_eq!("STANDARD".parse::<TrustTier>().unwrap(), TrustTier::Standard);
        assert_eq!("Standard".parse::<TrustTier>().unwrap(), TrustTier::Standard);
        assert_eq!("standard".parse::<TrustTier>().unwrap(), TrustTier::Standard);
    }

    #[test]
    fn trust_tier_from_str_returns_err_for_invalid() {
        assert!("invalid".parse::<TrustTier>().is_err());
        assert!("".parse::<TrustTier>().is_err());
    }

    #[test]
    fn trust_tier_is_sufficient_for_works() {
        assert!(TrustTier::Full.is_sufficient_for(TrustTier::Standard));
        assert!(TrustTier::Standard.is_sufficient_for(TrustTier::Standard));
        assert!(!TrustTier::Untrusted.is_sufficient_for(TrustTier::Standard));
    }

    #[test]
    fn verified_skill_creation() {
        let skill = test_skill();
        let uri = test_uri();
        let claims = test_claims();

        let verified = VerifiedSkill::new(skill, uri.clone(), claims, TrustTier::Standard);

        assert_eq!(verified.name(), "test-skill");
        assert_eq!(verified.trust_tier(), TrustTier::Standard);
        assert_eq!(verified.trust_root(), "acme.com");
        assert_eq!(verified.issuer(), "acme.com");
        assert!(verified.capabilities().contains(&"workflow/approval".to_string()));
    }

    #[test]
    fn verified_skill_is_not_expired() {
        let skill = test_skill();
        let uri = test_uri();
        let claims = test_claims();

        let verified = VerifiedSkill::new(skill, uri, claims, TrustTier::Standard);
        assert!(!verified.is_expired());
    }

    #[test]
    fn verified_skill_has_trust_tier_works() {
        let skill = test_skill();
        let uri = test_uri();
        let claims = test_claims();

        let verified = VerifiedSkill::new(skill, uri, claims, TrustTier::Standard);

        assert!(verified.has_trust_tier(TrustTier::Untrusted));
        assert!(verified.has_trust_tier(TrustTier::Standard));
        assert!(!verified.has_trust_tier(TrustTier::Elevated));
    }

    #[test]
    fn verified_skill_id_is_unique_per_instance() {
        let skill1 = test_skill();
        let skill2 = test_skill();
        let uri = test_uri();
        let claims = test_claims();

        let verified1 = VerifiedSkill::new(skill1, uri.clone(), claims.clone(), TrustTier::Standard);
        let verified2 = VerifiedSkill::new(skill2, uri, claims, TrustTier::Standard);

        assert_ne!(verified1.id(), verified2.id());
    }
}
