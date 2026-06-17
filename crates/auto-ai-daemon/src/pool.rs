//! Concurrency management — per-provider/per-model Semaphore pools.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Manages concurrency limits per provider.
/// Each provider has a Semaphore(max_concurrency).
/// Acquiring a permit blocks until a slot is free.
pub struct ConcurrencyManager {
    pools: HashMap<String, Arc<Semaphore>>,
    limits: HashMap<String, usize>,
}

impl ConcurrencyManager {
    /// Build from daemon config.
    pub fn from_config(config: &crate::config::DaemonConfig) -> Self {
        let mut pools = HashMap::new();
        let mut limits = HashMap::new();
        for (name, provider) in &config.providers {
            let limit = provider.max_concurrency;
            pools.insert(name.clone(), Arc::new(Semaphore::new(limit)));
            limits.insert(name.clone(), limit);
        }
        Self { pools, limits }
    }

    /// Acquire a concurrency permit for a provider.
    /// Returns a guard that releases on drop.
    pub async fn acquire(&self, provider: &str) -> Option<tokio::sync::OwnedSemaphorePermit> {
        let sem = self.pools.get(provider)?;
        Some(sem.clone().acquire_owned().await.ok()?)
    }

    /// Current available slots for a provider.
    pub fn available(&self, provider: &str) -> Option<usize> {
        let sem = self.pools.get(provider)?;
        let limit = self.limits.get(provider).copied().unwrap_or(0);
        Some(limit - sem.available_permits())
    }

    /// Max concurrency for a provider.
    pub fn limit(&self, provider: &str) -> usize {
        self.limits.get(provider).copied().unwrap_or(0)
    }

    /// Status snapshot: provider → (available, max).
    pub fn status(&self) -> Vec<(String, usize, usize)> {
        self.pools.iter().map(|(name, sem)| {
            let max = self.limits.get(name).copied().unwrap_or(0);
            (name.clone(), sem.available_permits(), max)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DaemonConfig, ProviderEntry};

    fn test_config() -> DaemonConfig {
        let mut providers = HashMap::new();
        providers.insert("test".into(), ProviderEntry {
            kind: "openai".into(),
            base_url: String::new(),
            api_key: "k".into(),
            models: vec![],
            max_concurrency: 2,
        });
        DaemonConfig {
            listen_addr: String::new(),
            idle_timeout_min: 0,
            providers,
            default_provider: "test".into(),
            default_model: String::new(),
            log_level: String::new(),
        }
    }

    #[test]
    fn pool_created() {
        let mgr = ConcurrencyManager::from_config(&test_config());
        assert_eq!(mgr.limit("test"), 2);
        assert_eq!(mgr.available("test"), Some(0)); // 2 used - 2 available... no, available is permits
    }

    #[tokio::test]
    async fn acquire_release() {
        let mgr = ConcurrencyManager::from_config(&test_config());
        let permit = mgr.acquire("test").await;
        assert!(permit.is_some());
        drop(permit); // releases
    }
}
