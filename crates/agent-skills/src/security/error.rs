//! Security-related error types for skill verification.
//!
//! This module defines errors that can occur during skill security
//! verification, including attestation validation, trust root checking,
//! capability enforcement, and cache operations.

use std::fmt;

use agent_uri_attestation::AttestationError;

/// Errors that can occur during skill security verification.
///
/// These errors cover all stages of the security verification process:
/// - Attestation token validation
/// - Trust root verification
/// - Capability enforcement
/// - Cache operations
/// - Registry communication
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillSecurityError {
    /// The skill file does not contain an agent-uri in its frontmatter.
    MissingAgentUri {
        /// Path to the skill that lacks an agent-uri.
        skill_path: String,
    },

    /// The agent-uri in the frontmatter is malformed.
    InvalidAgentUri {
        /// The invalid URI string.
        uri: String,
        /// Description of what is wrong.
        reason: String,
    },

    /// Failed to fetch attestation from the registry.
    AttestationFetchFailed {
        /// The agent-uri we attempted to fetch attestation for.
        agent_uri: String,
        /// The reason for the failure.
        reason: String,
    },

    /// The attestation token signature is invalid.
    InvalidSignature {
        /// The agent-uri whose attestation failed signature verification.
        agent_uri: String,
    },

    /// The attestation token has expired.
    TokenExpired {
        /// The agent-uri whose attestation has expired.
        agent_uri: String,
        /// When the token expired.
        expired_at: String,
    },

    /// The trust root in the attestation does not match the expected trust root.
    TrustRootMismatch {
        /// The trust root found in the token.
        token_root: String,
        /// The expected trust root from configuration.
        expected_root: String,
    },

    /// The issuer is not in the trusted roots set.
    UntrustedIssuer {
        /// The issuer that is not trusted.
        issuer: String,
    },

    /// The agent-uri in the token does not match the presented URI.
    UriMismatch {
        /// The URI in the attestation token.
        token_uri: String,
        /// The URI from the skill frontmatter.
        skill_uri: String,
    },

    /// The attested capabilities do not cover the required capability.
    InsufficientCapabilities {
        /// The capability path that was required.
        required: String,
        /// The capabilities that were attested.
        attested: Vec<String>,
    },

    /// The skill requires a trust tier higher than the current session allows.
    TrustTierInsufficient {
        /// The trust tier required by the skill.
        required_tier: String,
        /// The current session's trust tier.
        current_tier: String,
    },

    /// `OmniBOR` integrity check failed - the skill content has been modified.
    IntegrityCheckFailed {
        /// The expected `OmniBOR` artifact ID.
        expected_id: String,
        /// The computed artifact ID from current content.
        actual_id: String,
    },

    /// Cache operation failed.
    CacheError {
        /// Description of what went wrong.
        reason: String,
    },

    /// The attestation token format is invalid.
    InvalidTokenFormat {
        /// Description of the format error.
        reason: String,
    },

    /// The attestation claims are invalid or missing required fields.
    InvalidClaims {
        /// Description of what is wrong with the claims.
        reason: String,
    },

    /// A required configuration is missing.
    MissingConfiguration {
        /// Name of the missing configuration.
        config_name: String,
    },
}

impl fmt::Display for SkillSecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAgentUri { skill_path } => {
                write!(
                    f,
                    "skill at '{skill_path}' is missing 'agent-uri' in frontmatter metadata"
                )
            }
            Self::InvalidAgentUri { uri, reason } => {
                write!(f, "invalid agent-uri '{uri}': {reason}")
            }
            Self::AttestationFetchFailed { agent_uri, reason } => {
                write!(f, "failed to fetch attestation for '{agent_uri}': {reason}")
            }
            Self::InvalidSignature { agent_uri } => {
                write!(
                    f,
                    "attestation signature verification failed for '{agent_uri}'"
                )
            }
            Self::TokenExpired {
                agent_uri,
                expired_at,
            } => {
                write!(f, "attestation for '{agent_uri}' expired at {expired_at}")
            }
            Self::TrustRootMismatch {
                token_root,
                expected_root,
            } => {
                write!(
                    f,
                    "trust root mismatch: token has '{token_root}', expected '{expected_root}'"
                )
            }
            Self::UntrustedIssuer { issuer } => {
                write!(f, "issuer '{issuer}' is not in the trusted roots set")
            }
            Self::UriMismatch {
                token_uri,
                skill_uri,
            } => {
                write!(
                    f,
                    "agent-uri mismatch: token attests '{token_uri}', skill claims '{skill_uri}'"
                )
            }
            Self::InsufficientCapabilities { required, attested } => {
                write!(
                    f,
                    "capability '{required}' not covered by attested capabilities: [{}]",
                    attested.join(", ")
                )
            }
            Self::TrustTierInsufficient {
                required_tier,
                current_tier,
            } => {
                write!(
                    f,
                    "skill requires trust tier '{required_tier}', but session has '{current_tier}'"
                )
            }
            Self::IntegrityCheckFailed {
                expected_id,
                actual_id,
            } => {
                write!(
                    f,
                    "skill content integrity check failed: expected {expected_id}, computed {actual_id}"
                )
            }
            Self::CacheError { reason } => {
                write!(f, "attestation cache error: {reason}")
            }
            Self::InvalidTokenFormat { reason } => {
                write!(f, "invalid attestation token format: {reason}")
            }
            Self::InvalidClaims { reason } => {
                write!(f, "invalid attestation claims: {reason}")
            }
            Self::MissingConfiguration { config_name } => {
                write!(f, "missing required configuration: {config_name}")
            }
        }
    }
}

impl std::error::Error for SkillSecurityError {}

impl From<AttestationError> for SkillSecurityError {
    fn from(err: AttestationError) -> Self {
        match err {
            AttestationError::InvalidSignature => Self::InvalidSignature {
                agent_uri: "unknown".to_string(),
            },
            AttestationError::TokenExpired { expired_at } => Self::TokenExpired {
                agent_uri: "unknown".to_string(),
                expired_at,
            },
            AttestationError::TokenNotYetValid { valid_from } => Self::InvalidClaims {
                reason: format!("token not yet valid, valid from: {valid_from}"),
            },
            AttestationError::TrustRootMismatch {
                token_root,
                expected_root,
            } => Self::TrustRootMismatch {
                token_root,
                expected_root,
            },
            AttestationError::UntrustedIssuer { issuer } => Self::UntrustedIssuer { issuer },
            AttestationError::UriMismatch {
                token_uri,
                expected_uri,
            } => Self::UriMismatch {
                token_uri,
                skill_uri: expected_uri,
            },
            AttestationError::InsufficientCapabilities { required, attested } => {
                Self::InsufficientCapabilities { required, attested }
            }
            AttestationError::InvalidTokenFormat { reason } => Self::InvalidTokenFormat { reason },
            AttestationError::InvalidClaims { reason } => Self::InvalidClaims { reason },
            AttestationError::MissingField { field } => Self::InvalidClaims {
                reason: format!("missing required field: {field}"),
            },
            AttestationError::InvalidTtl => Self::InvalidClaims {
                reason: "invalid TTL value".to_string(),
            },
            AttestationError::MissingPublicKey { issuer } => Self::MissingConfiguration {
                config_name: format!("public key for issuer: {issuer}"),
            },
            AttestationError::InvalidKeyFormat { reason } => Self::InvalidClaims {
                reason: format!("invalid key format: {reason}"),
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn missing_agent_uri_display_includes_path() {
        let err = SkillSecurityError::MissingAgentUri {
            skill_path: "/path/to/skill.md".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/path/to/skill.md"));
        assert!(msg.contains("agent-uri"));
    }

    #[test]
    fn invalid_agent_uri_display_includes_uri_and_reason() {
        let err = SkillSecurityError::InvalidAgentUri {
            uri: "agent://bad".to_string(),
            reason: "missing path segments".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("agent://bad"));
        assert!(msg.contains("missing path segments"));
    }

    #[test]
    fn attestation_fetch_failed_display_is_informative() {
        let err = SkillSecurityError::AttestationFetchFailed {
            agent_uri: "agent://acme.com/test/skill_abc".to_string(),
            reason: "network timeout".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("acme.com"));
        assert!(msg.contains("network timeout"));
    }

    #[test]
    fn trust_root_mismatch_shows_both_roots() {
        let err = SkillSecurityError::TrustRootMismatch {
            token_root: "evil.com".to_string(),
            expected_root: "acme.com".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("evil.com"));
        assert!(msg.contains("acme.com"));
    }

    #[test]
    fn insufficient_capabilities_lists_attested() {
        let err = SkillSecurityError::InsufficientCapabilities {
            required: "admin/delete".to_string(),
            attested: vec!["read".to_string(), "write".to_string()],
        };
        let msg = err.to_string();
        assert!(msg.contains("admin/delete"));
        assert!(msg.contains("read"));
        assert!(msg.contains("write"));
    }

    #[test]
    fn trust_tier_insufficient_shows_both_tiers() {
        let err = SkillSecurityError::TrustTierInsufficient {
            required_tier: "elevated".to_string(),
            current_tier: "standard".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("elevated"));
        assert!(msg.contains("standard"));
    }

    #[test]
    fn integrity_check_failed_shows_ids() {
        let err = SkillSecurityError::IntegrityCheckFailed {
            expected_id: "gitoid:sha256:abc123".to_string(),
            actual_id: "gitoid:sha256:def456".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
    }

    #[test]
    fn from_attestation_error_maps_correctly() {
        let attestation_err = AttestationError::UntrustedIssuer {
            issuer: "malicious.com".to_string(),
        };
        let skill_err: SkillSecurityError = attestation_err.into();
        assert!(matches!(
            skill_err,
            SkillSecurityError::UntrustedIssuer { issuer } if issuer == "malicious.com"
        ));
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SkillSecurityError>();
    }
}
