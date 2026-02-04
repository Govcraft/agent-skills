//! HTTP client for fetching attestations from a skill registry.
//!
//! This module provides a client for communicating with a skill registry
//! service (like `TalonHub`) to fetch attestation tokens for skills.

use std::time::Duration;

use crate::security::error::SkillSecurityError;

/// Configuration for the registry client.
#[derive(Debug, Clone)]
pub struct RegistryClientConfig {
    /// Base URL of the registry service.
    pub base_url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Whether to verify TLS certificates.
    pub verify_tls: bool,
    /// User agent string for requests.
    pub user_agent: String,
}

impl Default for RegistryClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://registry.agentskills.io".to_string(),
            timeout: Duration::from_secs(30),
            verify_tls: true,
            user_agent: format!("agent-skills/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl RegistryClientConfig {
    /// Creates a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base URL for the registry.
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the request timeout.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets whether to verify TLS certificates.
    #[must_use]
    pub const fn verify_tls(mut self, verify: bool) -> Self {
        self.verify_tls = verify;
        self
    }

    /// Sets the user agent string.
    #[must_use]
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = agent.into();
        self
    }
}

/// Response from the registry containing an attestation token.
#[derive(Debug, Clone)]
pub struct AttestationResponse {
    /// The PASETO attestation token.
    pub token: String,
    /// The trust root that issued the attestation.
    pub issuer: String,
    /// When the attestation was issued (ISO 8601).
    pub issued_at: String,
    /// When the attestation expires (ISO 8601).
    pub expires_at: String,
}

/// Client for fetching attestations from a skill registry.
///
/// This client communicates with a registry service to retrieve
/// PASETO attestation tokens for skills identified by their agent URIs.
///
/// # Example
///
/// ```ignore
/// use agent_skills::security::{RegistryClient, RegistryClientConfig};
///
/// let config = RegistryClientConfig::new()
///     .base_url("https://registry.example.com");
///
/// let client = RegistryClient::new(config);
///
/// // Fetch attestation for a skill
/// let response = client.fetch_attestation(
///     "agent://acme.com/workflow/approval/skill_abc"
/// ).await?;
///
/// println!("Token: {}", response.token);
/// ```
#[derive(Debug, Clone)]
pub struct RegistryClient {
    /// Configuration for this client.
    config: RegistryClientConfig,
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new(RegistryClientConfig::default())
    }
}

impl RegistryClient {
    /// Creates a new registry client with the given configuration.
    #[must_use]
    pub const fn new(config: RegistryClientConfig) -> Self {
        Self { config }
    }

    /// Returns the base URL of the registry.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Fetches an attestation token for the given agent URI.
    ///
    /// # Arguments
    ///
    /// * `agent_uri` - The agent URI to fetch attestation for
    ///
    /// # Errors
    ///
    /// Returns `SkillSecurityError::AttestationFetchFailed` if the request fails.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. In production, this would
    /// make actual HTTP requests to the registry service.
    pub fn fetch_attestation(
        &self,
        agent_uri: &str,
    ) -> Result<AttestationResponse, SkillSecurityError> {
        // Build the URL for the attestation endpoint
        let _url = format!(
            "{}/v1/attestations/{}",
            self.config.base_url,
            urlencoding::encode(agent_uri)
        );

        // Placeholder: In production, this would make an HTTP request
        // using reqwest or similar. For now, return an error indicating
        // the registry is not available.
        Err(SkillSecurityError::AttestationFetchFailed {
            agent_uri: agent_uri.to_string(),
            reason: "registry client requires async runtime (use fetch_attestation_async)"
                .to_string(),
        })
    }

    /// Validates that the registry is reachable.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry health check fails.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation.
    pub const fn health_check(&self) -> Result<(), SkillSecurityError> {
        // Placeholder: In production, this would ping the registry's
        // health endpoint
        Ok(())
    }
}

/// Pure function to build the attestation URL.
///
/// This function is extracted to make URL construction testable
/// without making HTTP requests.
///
/// # Arguments
///
/// * `base_url` - The base URL of the registry
/// * `agent_uri` - The agent URI to fetch attestation for
///
/// # Returns
///
/// The full URL for the attestation endpoint.
#[must_use]
pub fn build_attestation_url(base_url: &str, agent_uri: &str) -> String {
    format!(
        "{}/v1/attestations/{}",
        base_url.trim_end_matches('/'),
        urlencoding::encode(agent_uri)
    )
}

/// Pure function to validate an attestation response.
///
/// This function checks that all required fields are present and
/// properly formatted.
///
/// # Arguments
///
/// * `response` - The attestation response to validate
///
/// # Errors
///
/// Returns an error if the response is invalid.
pub fn validate_attestation_response(
    response: &AttestationResponse,
) -> Result<(), SkillSecurityError> {
    if response.token.is_empty() {
        return Err(SkillSecurityError::InvalidTokenFormat {
            reason: "attestation token is empty".to_string(),
        });
    }

    if !response.token.starts_with("v4.public.") {
        return Err(SkillSecurityError::InvalidTokenFormat {
            reason: "attestation token must be PASETO v4.public format".to_string(),
        });
    }

    if response.issuer.is_empty() {
        return Err(SkillSecurityError::InvalidClaims {
            reason: "issuer is empty".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_works() {
        let config = RegistryClientConfig::new()
            .base_url("https://custom.registry.io")
            .timeout(Duration::from_secs(60))
            .verify_tls(false)
            .user_agent("test-agent/1.0");

        assert_eq!(config.base_url, "https://custom.registry.io");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(!config.verify_tls);
        assert_eq!(config.user_agent, "test-agent/1.0");
    }

    #[test]
    fn client_creation_works() {
        let config = RegistryClientConfig::new().base_url("https://example.com");

        let client = RegistryClient::new(config);

        assert_eq!(client.base_url(), "https://example.com");
    }

    #[test]
    fn build_attestation_url_encodes_uri() {
        let url = build_attestation_url(
            "https://registry.example.com",
            "agent://acme.com/test/skill_abc",
        );

        assert!(url.starts_with("https://registry.example.com/v1/attestations/"));
        assert!(url.contains("agent%3A%2F%2Facme.com"));
    }

    #[test]
    fn build_attestation_url_handles_trailing_slash() {
        let url1 = build_attestation_url("https://example.com", "agent://test");
        let url2 = build_attestation_url("https://example.com/", "agent://test");

        assert_eq!(url1, url2);
    }

    #[test]
    fn validate_response_rejects_empty_token() {
        let response = AttestationResponse {
            token: String::new(),
            issuer: "acme.com".to_string(),
            issued_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-01-02T00:00:00Z".to_string(),
        };

        let result = validate_attestation_response(&response);
        assert!(matches!(
            result,
            Err(SkillSecurityError::InvalidTokenFormat { .. })
        ));
    }

    #[test]
    fn validate_response_rejects_non_paseto_token() {
        let response = AttestationResponse {
            token: "not-a-paseto-token".to_string(),
            issuer: "acme.com".to_string(),
            issued_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-01-02T00:00:00Z".to_string(),
        };

        let result = validate_attestation_response(&response);
        assert!(matches!(
            result,
            Err(SkillSecurityError::InvalidTokenFormat { .. })
        ));
    }

    #[test]
    fn validate_response_rejects_empty_issuer() {
        let response = AttestationResponse {
            token: "v4.public.valid-token".to_string(),
            issuer: String::new(),
            issued_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-01-02T00:00:00Z".to_string(),
        };

        let result = validate_attestation_response(&response);
        assert!(matches!(
            result,
            Err(SkillSecurityError::InvalidClaims { .. })
        ));
    }

    #[test]
    fn validate_response_accepts_valid_response() {
        let response = AttestationResponse {
            token: "v4.public.valid-token".to_string(),
            issuer: "acme.com".to_string(),
            issued_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-01-02T00:00:00Z".to_string(),
        };

        let result = validate_attestation_response(&response);
        assert!(result.is_ok());
    }

    #[test]
    fn health_check_succeeds() {
        let client = RegistryClient::default();
        assert!(client.health_check().is_ok());
    }

    #[test]
    fn default_config_has_sensible_values() {
        let config = RegistryClientConfig::default();

        assert!(!config.base_url.is_empty());
        assert!(config.timeout.as_secs() > 0);
        assert!(config.verify_tls);
        assert!(config.user_agent.contains("agent-skills"));
    }
}
