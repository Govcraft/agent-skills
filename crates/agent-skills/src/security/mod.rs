//! Security features for skill verification and access control.
//!
//! This module provides cryptographic verification of skills using agent-uri
//! identity and PASETO attestation tokens. It enables:
//!
//! - **Identity verification** - Skills are identified by agent URIs
//! - **Attestation validation** - PASETO v4.public tokens signed by trusted roots
//! - **Capability-based access** - Skills declare required capabilities
//! - **Trust tier enforcement** - Different capabilities require different trust levels
//!
//! # Quick Start
//!
//! ```ignore
//! use agent_skills::security::{
//!     SecureSkillRegistry,
//!     SecureSkillRegistryConfig,
//!     TrustTier,
//! };
//! use agent_uri_attestation::VerifyingKey;
//!
//! // Create a secure registry
//! let config = SecureSkillRegistryConfig::new()
//!     .default_trust_tier(TrustTier::Standard);
//! let mut registry = SecureSkillRegistry::new(config);
//!
//! // Add trusted attestation roots
//! registry.add_trusted_root("acme.com", verifying_key);
//!
//! // Load a verified skill
//! let skill = registry.load_verified(Path::new("/path/to/skill"))?;
//!
//! println!("Loaded: {} (trust: {})", skill.name(), skill.trust_tier());
//! println!("Capabilities: {:?}", skill.capabilities());
//! ```
//!
//! # Skill Frontmatter
//!
//! To be verifiable, skills must include an `agent-uri` in their frontmatter metadata:
//!
//! ```yaml
//! ---
//! name: my-skill
//! description: Does something useful.
//! metadata:
//!   agent-uri: "agent://acme.com/workflow/approval/skill_01h455vb4pex5vsknk084sn02q"
//! ---
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    SecureSkillRegistry                       │
//! │  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐    │
//! │  │   Verifier  │  │    Cache    │  │  RegistryClient  │    │
//! │  │  (PASETO)   │  │ (Attestn.)  │  │    (HTTP)        │    │
//! │  └──────┬──────┘  └──────┬──────┘  └────────┬─────────┘    │
//! │         │                │                  │               │
//! │         └────────────────┼──────────────────┘               │
//! │                          │                                  │
//! │                    load_verified()                          │
//! │                          │                                  │
//! │                          ▼                                  │
//! │                   VerifiedSkill                             │
//! │            (Skill + AgentUri + Claims + TrustTier)          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Trust Tiers
//!
//! | Tier | Description | Example Capabilities |
//! |------|-------------|---------------------|
//! | `Untrusted` | No verification | (none) |
//! | `Standard` | Basic operations | filesystem/read, git |
//! | `Elevated` | Code execution | bash, browser, llm |
//! | `Full` | System access | admin/*, system/* |

mod cache;
mod capabilities;
mod error;
mod registry;
mod registry_client;
mod verified;

// Re-export main types
pub use cache::{AttestationCache, CacheConfig, CacheStats};
pub use capabilities::{
    capability_covers_path, capability_to_trust_tier, max_trust_tier_for_capabilities,
    tool_to_capability, validate_tool_capabilities, validate_trust_tier,
};
pub use error::SkillSecurityError;
pub use registry::{SecureSkillRegistry, SecureSkillRegistryConfig, SkillLoadError};
pub use registry_client::{
    build_attestation_url, validate_attestation_response, AttestationResponse, RegistryClient,
    RegistryClientConfig,
};
pub use verified::{ParseTrustTierError, SkillId, TrustTier, VerifiedSkill};
