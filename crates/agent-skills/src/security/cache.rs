//! Attestation caching with TTL-based expiration.
//!
//! This module provides a cache for verified attestations to avoid
//! re-fetching and re-verifying attestations for frequently used skills.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use agent_uri_attestation::AttestationClaims;
use chrono::{DateTime, Utc};

/// Configuration for the attestation cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries to store.
    pub max_entries: usize,
    /// Default TTL for cached entries.
    pub default_ttl: Duration,
    /// Whether to automatically evict expired entries on access.
    pub auto_evict: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(3600), // 1 hour
            auto_evict: true,
        }
    }
}

impl CacheConfig {
    /// Creates a new cache configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of entries.
    #[must_use]
    pub const fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Sets the default TTL for cached entries.
    #[must_use]
    pub const fn default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Sets whether to automatically evict expired entries.
    #[must_use]
    pub const fn auto_evict(mut self, auto: bool) -> Self {
        self.auto_evict = auto;
        self
    }
}

/// A cached attestation entry with expiration tracking.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached attestation claims.
    claims: AttestationClaims,
    /// When this entry was cached.
    cached_at: DateTime<Utc>,
    /// When this entry expires (from cache, not token expiration).
    cache_expires_at: DateTime<Utc>,
    /// Number of times this entry has been accessed.
    access_count: u64,
}

impl CacheEntry {
    /// Creates a new cache entry.
    fn new(claims: AttestationClaims, ttl: Duration) -> Self {
        let now = Utc::now();
        let cache_expires_at = now
            + chrono::Duration::from_std(ttl).unwrap_or_else(|_| chrono::Duration::hours(1));
        Self {
            claims,
            cached_at: now,
            cache_expires_at,
            access_count: 0,
        }
    }

    /// Returns true if this cache entry has expired.
    fn is_cache_expired(&self) -> bool {
        Utc::now() >= self.cache_expires_at
    }

    /// Returns true if the underlying token has expired.
    fn is_token_expired(&self) -> bool {
        self.claims.is_expired()
    }

    /// Increments the access count and returns a clone of the claims.
    fn access(&mut self) -> AttestationClaims {
        self.access_count = self.access_count.saturating_add(1);
        self.claims.clone()
    }
}

/// Thread-safe cache for verified attestations.
///
/// This cache stores attestation claims keyed by agent URI, with automatic
/// expiration and LRU-style eviction when the cache is full.
///
/// # Example
///
/// ```
/// use agent_skills::security::AttestationCache;
/// use agent_uri_attestation::AttestationClaims;
/// use std::time::Duration;
///
/// let cache = AttestationCache::new();
///
/// // Cache an attestation
/// let claims = AttestationClaims::builder()
///     .agent_uri("agent://acme.com/test/skill_abc")
///     .issuer("acme.com")
///     .ttl(Duration::from_secs(3600))
///     .build()
///     .unwrap();
///
/// cache.insert("agent://acme.com/test/skill_abc", claims.clone());
///
/// // Retrieve from cache
/// if let Some(cached) = cache.get("agent://acme.com/test/skill_abc") {
///     assert_eq!(cached.agent_uri, claims.agent_uri);
/// }
/// ```
#[derive(Debug)]
pub struct AttestationCache {
    /// Configuration for this cache.
    config: CacheConfig,
    /// The cached entries, keyed by agent URI.
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl Clone for AttestationCache {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            entries: Arc::clone(&self.entries),
        }
    }
}

impl Default for AttestationCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AttestationCache {
    /// Creates a new cache with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Creates a new cache with the given configuration.
    #[must_use]
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inserts an attestation into the cache.
    ///
    /// If the cache is at capacity, the oldest entry will be evicted.
    pub fn insert(&self, agent_uri: &str, claims: AttestationClaims) {
        self.insert_with_ttl(agent_uri, claims, self.config.default_ttl);
    }

    /// Inserts an attestation with a custom TTL.
    pub fn insert_with_ttl(&self, agent_uri: &str, claims: AttestationClaims, ttl: Duration) {
        let entry = CacheEntry::new(claims, ttl);

        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Evict if at capacity
        if entries.len() >= self.config.max_entries && !entries.contains_key(agent_uri) {
            evict_oldest_entry(&mut entries);
        }

        entries.insert(agent_uri.to_string(), entry);
    }

    /// Retrieves an attestation from the cache.
    ///
    /// Returns `None` if the entry doesn't exist, has expired, or the
    /// underlying token has expired.
    #[must_use]
    pub fn get(&self, agent_uri: &str) -> Option<AttestationClaims> {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Check if entry exists and whether it's expired
        let (is_cache_expired, is_token_expired) = {
            let entry = entries.get(agent_uri)?;
            (entry.is_cache_expired(), entry.is_token_expired())
        };

        // Handle cache expiration
        if is_cache_expired {
            if self.config.auto_evict {
                entries.remove(agent_uri);
            }
            return None;
        }

        // Handle token expiration
        if is_token_expired {
            if self.config.auto_evict {
                entries.remove(agent_uri);
            }
            return None;
        }

        // Entry is valid, access it
        entries.get_mut(agent_uri).map(CacheEntry::access)
    }

    /// Returns true if the cache contains a valid entry for the given URI.
    #[must_use]
    pub fn contains(&self, agent_uri: &str) -> bool {
        self.get(agent_uri).is_some()
    }

    /// Removes an entry from the cache.
    #[must_use]
    pub fn remove(&self, agent_uri: &str) -> Option<AttestationClaims> {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        entries.remove(agent_uri).map(|e| e.claims)
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        entries.clear();
    }

    /// Returns the number of entries in the cache.
    ///
    /// Note: This count may include expired entries if `auto_evict` is disabled.
    #[must_use]
    pub fn len(&self) -> usize {
        let entries = match self.entries.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        entries.len()
    }

    /// Returns true if the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes all expired entries from the cache.
    ///
    /// This is useful when `auto_evict` is disabled and you want to
    /// manually clean up the cache.
    pub fn evict_expired(&self) {
        let mut entries = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        entries.retain(|_, entry| !entry.is_cache_expired() && !entry.is_token_expired());
    }

    /// Returns cache statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        let entries = match self.entries.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let total = entries.len();
        let expired = entries
            .values()
            .filter(|e| e.is_cache_expired() || e.is_token_expired())
            .count();
        let total_accesses: u64 = entries.values().map(|e| e.access_count).sum();
        let max = self.config.max_entries;

        drop(entries);

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            total_accesses,
            max_entries: max,
        }
    }
}

/// Evicts the oldest entry from the cache.
fn evict_oldest_entry(entries: &mut HashMap<String, CacheEntry>) {
    // Find the oldest entry by cached_at time
    let oldest_key = entries
        .iter()
        .min_by_key(|(_, entry)| entry.cached_at)
        .map(|(key, _)| key.clone());

    if let Some(key) = oldest_key {
        entries.remove(&key);
    }
}

/// Statistics about the cache state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    /// Total number of entries in the cache.
    pub total_entries: usize,
    /// Number of expired entries (if `auto_evict` is disabled).
    pub expired_entries: usize,
    /// Total number of accesses across all entries.
    pub total_accesses: u64,
    /// Maximum number of entries allowed.
    pub max_entries: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_claims(uri: &str) -> AttestationClaims {
        AttestationClaims::builder()
            .agent_uri(uri)
            .issuer("acme.com")
            .ttl(Duration::from_secs(3600))
            .build()
            .unwrap()
    }

    #[test]
    fn cache_insert_and_get() {
        let cache = AttestationCache::new();
        let claims = test_claims("agent://acme.com/test/skill_abc");

        cache.insert("agent://acme.com/test/skill_abc", claims.clone());

        let retrieved = cache.get("agent://acme.com/test/skill_abc");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().agent_uri, claims.agent_uri);
    }

    #[test]
    fn cache_returns_none_for_missing() {
        let cache = AttestationCache::new();
        assert!(cache.get("agent://nonexistent").is_none());
    }

    #[test]
    fn cache_contains_works() {
        let cache = AttestationCache::new();
        let claims = test_claims("agent://acme.com/test/skill_abc");

        assert!(!cache.contains("agent://acme.com/test/skill_abc"));

        cache.insert("agent://acme.com/test/skill_abc", claims);

        assert!(cache.contains("agent://acme.com/test/skill_abc"));
    }

    #[test]
    fn cache_remove_works() {
        let cache = AttestationCache::new();
        let claims = test_claims("agent://acme.com/test/skill_abc");

        cache.insert("agent://acme.com/test/skill_abc", claims.clone());
        assert!(cache.contains("agent://acme.com/test/skill_abc"));

        let removed = cache.remove("agent://acme.com/test/skill_abc");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().agent_uri, claims.agent_uri);
        assert!(!cache.contains("agent://acme.com/test/skill_abc"));
    }

    #[test]
    fn cache_clear_works() {
        let cache = AttestationCache::new();
        cache.insert(
            "agent://acme.com/test/skill_1",
            test_claims("agent://acme.com/test/skill_1"),
        );
        cache.insert(
            "agent://acme.com/test/skill_2",
            test_claims("agent://acme.com/test/skill_2"),
        );

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert!(cache.is_empty());
    }

    #[test]
    fn cache_respects_max_entries() {
        let config = CacheConfig::new().max_entries(2);
        let cache = AttestationCache::with_config(config);

        cache.insert(
            "agent://acme.com/test/skill_1",
            test_claims("agent://acme.com/test/skill_1"),
        );
        cache.insert(
            "agent://acme.com/test/skill_2",
            test_claims("agent://acme.com/test/skill_2"),
        );
        cache.insert(
            "agent://acme.com/test/skill_3",
            test_claims("agent://acme.com/test/skill_3"),
        );

        assert_eq!(cache.len(), 2);
        // The newest should be present
        assert!(cache.contains("agent://acme.com/test/skill_3"));
    }

    #[test]
    fn cache_evicts_expired_entries() {
        let config = CacheConfig::new()
            .default_ttl(Duration::from_millis(1))
            .auto_evict(true);
        let cache = AttestationCache::with_config(config);

        cache.insert(
            "agent://acme.com/test/skill_1",
            test_claims("agent://acme.com/test/skill_1"),
        );

        // Wait for cache TTL to expire
        std::thread::sleep(Duration::from_millis(10));

        assert!(cache.get("agent://acme.com/test/skill_1").is_none());
    }

    #[test]
    fn cache_stats_are_accurate() {
        let cache = AttestationCache::new();

        cache.insert(
            "agent://acme.com/test/skill_1",
            test_claims("agent://acme.com/test/skill_1"),
        );
        cache.insert(
            "agent://acme.com/test/skill_2",
            test_claims("agent://acme.com/test/skill_2"),
        );

        // Access one entry
        let _ = cache.get("agent://acme.com/test/skill_1");

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.expired_entries, 0);
        assert_eq!(stats.total_accesses, 1);
    }

    #[test]
    fn cache_is_clone() {
        let cache1 = AttestationCache::new();
        cache1.insert(
            "agent://acme.com/test/skill_1",
            test_claims("agent://acme.com/test/skill_1"),
        );

        let cache2 = cache1.clone();

        // Both should see the same data
        assert!(cache1.contains("agent://acme.com/test/skill_1"));
        assert!(cache2.contains("agent://acme.com/test/skill_1"));

        // Insert via clone should be visible to original
        cache2.insert(
            "agent://acme.com/test/skill_2",
            test_claims("agent://acme.com/test/skill_2"),
        );
        assert!(cache1.contains("agent://acme.com/test/skill_2"));
    }

    #[test]
    fn cache_config_builder_works() {
        let config = CacheConfig::new()
            .max_entries(500)
            .default_ttl(Duration::from_secs(1800))
            .auto_evict(false);

        assert_eq!(config.max_entries, 500);
        assert_eq!(config.default_ttl, Duration::from_secs(1800));
        assert!(!config.auto_evict);
    }
}
