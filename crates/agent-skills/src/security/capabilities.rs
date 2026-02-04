//! Capability mapping and validation for skill security.
//!
//! This module provides functions to map tools to capability paths and
//! validate that skills have the required capabilities for their operations.

use agent_uri::CapabilityPath;
use agent_uri_attestation::capability_covers;

use crate::security::error::SkillSecurityError;
use crate::security::verified::TrustTier;

/// Maps a tool name to its required capability path.
///
/// This function translates tool names (as used in skill frontmatter's
/// `allowed-tools` field) to the capability paths required to use them.
///
/// # Arguments
///
/// * `tool_name` - The name of the tool (e.g., "Bash", "Read", "Write")
///
/// # Returns
///
/// The capability path required to use the tool, or `None` if the tool
/// is unknown.
///
/// # Example
///
/// ```
/// use agent_skills::security::tool_to_capability;
///
/// let cap = tool_to_capability("Bash").unwrap();
/// assert_eq!(cap.as_str(), "tools/bash");
///
/// let cap = tool_to_capability("Read").unwrap();
/// assert_eq!(cap.as_str(), "tools/filesystem/read");
/// ```
#[must_use]
pub fn tool_to_capability(tool_name: &str) -> Option<CapabilityPath> {
    let path = match tool_name {
        // Shell/execution tools
        "Bash" | "Shell" => "tools/bash",

        // Filesystem tools - read access
        "Read" | "Glob" | "Grep" => "tools/filesystem/read",

        // Filesystem tools - write access
        "Write" | "Edit" => "tools/filesystem/write",

        // Network tools
        "WebFetch" => "tools/network/http",
        "WebSearch" => "tools/network/search",

        // Git tools
        "Git" => "tools/git",

        // Browser tools
        "Browser" => "tools/browser",

        // Notebook tools
        "NotebookEdit" => "tools/notebook",

        // Task management
        "Task" => "tools/task",

        // Agent/LLM tools
        "Agent" => "tools/agent",
        "LLM" => "tools/llm",

        // Unknown tool
        _ => return None,
    };

    CapabilityPath::parse(path).ok()
}

/// Maps a capability path to its required trust tier.
///
/// Different capabilities require different trust levels. This function
/// determines the minimum trust tier needed for a given capability.
///
/// # Arguments
///
/// * `capability` - The capability path to check
///
/// # Returns
///
/// The minimum trust tier required for the capability.
///
/// # Example
///
/// ```
/// use agent_uri::CapabilityPath;
/// use agent_skills::security::{capability_to_trust_tier, TrustTier};
///
/// let cap = CapabilityPath::parse("tools/bash").unwrap();
/// assert_eq!(capability_to_trust_tier(&cap), TrustTier::Elevated);
///
/// let cap = CapabilityPath::parse("tools/filesystem/read").unwrap();
/// assert_eq!(capability_to_trust_tier(&cap), TrustTier::Standard);
/// ```
#[must_use]
pub fn capability_to_trust_tier(capability: &CapabilityPath) -> TrustTier {
    let path = capability.as_str();

    // Elevated trust capabilities (can execute arbitrary code or access network)
    if path.starts_with("tools/bash")
        || path.starts_with("tools/shell")
        || path.starts_with("tools/agent")
        || path.starts_with("tools/llm")
        || path.starts_with("tools/browser")
    {
        return TrustTier::Elevated;
    }

    // Full trust capabilities (system-level access)
    if path.starts_with("system/") || path.starts_with("admin/") {
        return TrustTier::Full;
    }

    // Standard trust capabilities (filesystem, git, etc.)
    if path.starts_with("tools/filesystem")
        || path.starts_with("tools/git")
        || path.starts_with("tools/network")
        || path.starts_with("tools/notebook")
        || path.starts_with("tools/task")
    {
        return TrustTier::Standard;
    }

    // Unknown capabilities default to standard
    TrustTier::Standard
}

/// Determines the maximum trust tier required by a set of capabilities.
///
/// # Arguments
///
/// * `capabilities` - The capability strings from an attestation
///
/// # Returns
///
/// The highest trust tier required by any of the capabilities.
///
/// # Example
///
/// ```
/// use agent_skills::security::{max_trust_tier_for_capabilities, TrustTier};
///
/// let caps = vec!["tools/filesystem/read".to_string(), "tools/bash".to_string()];
/// assert_eq!(max_trust_tier_for_capabilities(&caps), TrustTier::Elevated);
/// ```
#[must_use]
pub fn max_trust_tier_for_capabilities(capabilities: &[String]) -> TrustTier {
    capabilities
        .iter()
        .filter_map(|cap| CapabilityPath::parse(cap).ok())
        .map(|cap| capability_to_trust_tier(&cap))
        .max()
        .unwrap_or(TrustTier::Untrusted)
}

/// Validates that the attested capabilities cover all required tools.
///
/// # Arguments
///
/// * `attested_capabilities` - The capabilities from the attestation
/// * `required_tools` - The tools the skill wants to use
///
/// # Returns
///
/// `Ok(())` if all tools are covered, or an error describing missing capabilities.
///
/// # Errors
///
/// Returns [`SkillSecurityError::InsufficientCapabilities`] if a required tool's
/// capability is not covered by the attested capabilities.
///
/// # Example
///
/// ```
/// use agent_skills::security::validate_tool_capabilities;
///
/// let attested = vec!["tools/filesystem".to_string()];
/// let tools = vec!["Read", "Write"];
///
/// assert!(validate_tool_capabilities(&attested, &tools).is_ok());
///
/// let tools_with_bash = vec!["Read", "Bash"];
/// assert!(validate_tool_capabilities(&attested, &tools_with_bash).is_err());
/// ```
pub fn validate_tool_capabilities(
    attested_capabilities: &[String],
    required_tools: &[&str],
) -> Result<(), SkillSecurityError> {
    for tool in required_tools {
        if let Some(required_cap) = tool_to_capability(tool)
            && !capability_covers(attested_capabilities, &required_cap)
        {
            return Err(SkillSecurityError::InsufficientCapabilities {
                required: required_cap.to_string(),
                attested: attested_capabilities.to_vec(),
            });
        }
        // Unknown tools are allowed (they may be custom or future tools)
    }
    Ok(())
}

/// Validates that the current trust tier is sufficient for the required capabilities.
///
/// # Arguments
///
/// * `current_tier` - The current session's trust tier
/// * `capabilities` - The capabilities being requested
///
/// # Returns
///
/// `Ok(())` if the trust tier is sufficient, or an error if not.
///
/// # Errors
///
/// Returns [`SkillSecurityError::TrustTierInsufficient`] if the current trust tier
/// is lower than the required tier for the given capabilities.
///
/// # Example
///
/// ```
/// use agent_skills::security::{validate_trust_tier, TrustTier};
///
/// let caps = vec!["tools/filesystem/read".to_string()];
///
/// assert!(validate_trust_tier(TrustTier::Standard, &caps).is_ok());
/// assert!(validate_trust_tier(TrustTier::Untrusted, &caps).is_err());
/// ```
pub fn validate_trust_tier(
    current_tier: TrustTier,
    capabilities: &[String],
) -> Result<(), SkillSecurityError> {
    let required_tier = max_trust_tier_for_capabilities(capabilities);

    if current_tier.is_sufficient_for(required_tier) {
        Ok(())
    } else {
        Err(SkillSecurityError::TrustTierInsufficient {
            required_tier: required_tier.to_string(),
            current_tier: current_tier.to_string(),
        })
    }
}

/// Pure function to check if a capability covers a required path.
///
/// This is a wrapper around `agent_uri_attestation::capability_covers`
/// that takes a string path and parses it.
///
/// # Arguments
///
/// * `attested` - The attested capability strings
/// * `required_path` - The required capability path as a string
///
/// # Returns
///
/// `true` if any attested capability covers the required path.
#[must_use]
pub fn capability_covers_path(attested: &[String], required_path: &str) -> bool {
    CapabilityPath::parse(required_path)
        .is_ok_and(|cap| capability_covers(attested, &cap))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn tool_to_capability_maps_bash() {
        let cap = tool_to_capability("Bash").unwrap();
        assert_eq!(cap.as_str(), "tools/bash");
    }

    #[test]
    fn tool_to_capability_maps_filesystem_tools() {
        assert_eq!(tool_to_capability("Read").unwrap().as_str(), "tools/filesystem/read");
        assert_eq!(tool_to_capability("Write").unwrap().as_str(), "tools/filesystem/write");
        assert_eq!(tool_to_capability("Edit").unwrap().as_str(), "tools/filesystem/write");
        assert_eq!(tool_to_capability("Glob").unwrap().as_str(), "tools/filesystem/read");
        assert_eq!(tool_to_capability("Grep").unwrap().as_str(), "tools/filesystem/read");
    }

    #[test]
    fn tool_to_capability_maps_network_tools() {
        assert_eq!(tool_to_capability("WebFetch").unwrap().as_str(), "tools/network/http");
        assert_eq!(tool_to_capability("WebSearch").unwrap().as_str(), "tools/network/search");
    }

    #[test]
    fn tool_to_capability_returns_none_for_unknown() {
        assert!(tool_to_capability("UnknownTool").is_none());
        assert!(tool_to_capability("").is_none());
    }

    #[test]
    fn capability_to_trust_tier_bash_is_elevated() {
        let cap = CapabilityPath::parse("tools/bash").unwrap();
        assert_eq!(capability_to_trust_tier(&cap), TrustTier::Elevated);
    }

    #[test]
    fn capability_to_trust_tier_filesystem_is_standard() {
        let cap = CapabilityPath::parse("tools/filesystem/read").unwrap();
        assert_eq!(capability_to_trust_tier(&cap), TrustTier::Standard);
    }

    #[test]
    fn capability_to_trust_tier_admin_is_full() {
        let cap = CapabilityPath::parse("admin/settings").unwrap();
        assert_eq!(capability_to_trust_tier(&cap), TrustTier::Full);
    }

    #[test]
    fn max_trust_tier_returns_highest() {
        let caps = vec![
            "tools/filesystem/read".to_string(),
            "tools/bash".to_string(),
        ];
        assert_eq!(max_trust_tier_for_capabilities(&caps), TrustTier::Elevated);
    }

    #[test]
    fn max_trust_tier_returns_untrusted_for_empty() {
        let caps: Vec<String> = vec![];
        assert_eq!(max_trust_tier_for_capabilities(&caps), TrustTier::Untrusted);
    }

    #[test]
    fn validate_tool_capabilities_succeeds_when_covered() {
        let attested = vec!["tools/filesystem".to_string()];
        let tools = vec!["Read", "Write"];

        assert!(validate_tool_capabilities(&attested, &tools).is_ok());
    }

    #[test]
    fn validate_tool_capabilities_fails_when_not_covered() {
        let attested = vec!["tools/filesystem/read".to_string()];
        let tools = vec!["Bash"];

        let result = validate_tool_capabilities(&attested, &tools);
        assert!(matches!(
            result,
            Err(SkillSecurityError::InsufficientCapabilities { .. })
        ));
    }

    #[test]
    fn validate_tool_capabilities_ignores_unknown_tools() {
        let attested = vec!["tools/filesystem".to_string()];
        let tools = vec!["Read", "CustomTool"];

        // Unknown tools are allowed - they might be custom tools
        assert!(validate_tool_capabilities(&attested, &tools).is_ok());
    }

    #[test]
    fn validate_trust_tier_succeeds_when_sufficient() {
        let caps = vec!["tools/filesystem/read".to_string()];
        assert!(validate_trust_tier(TrustTier::Standard, &caps).is_ok());
        assert!(validate_trust_tier(TrustTier::Elevated, &caps).is_ok());
    }

    #[test]
    fn validate_trust_tier_fails_when_insufficient() {
        let caps = vec!["tools/bash".to_string()];
        let result = validate_trust_tier(TrustTier::Standard, &caps);

        assert!(matches!(
            result,
            Err(SkillSecurityError::TrustTierInsufficient { .. })
        ));
    }

    #[test]
    fn capability_covers_path_works() {
        let attested = vec!["tools/filesystem".to_string()];

        assert!(capability_covers_path(&attested, "tools/filesystem/read"));
        assert!(capability_covers_path(&attested, "tools/filesystem/write"));
        assert!(!capability_covers_path(&attested, "tools/bash"));
    }

    #[test]
    fn capability_covers_path_handles_invalid_path() {
        let attested = vec!["tools/filesystem".to_string()];
        assert!(!capability_covers_path(&attested, "")); // Empty is invalid
    }
}
