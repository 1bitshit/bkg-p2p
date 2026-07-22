use crate::documentation::types::*;
use anyhow::Result;
use std::collections::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// Content-addressed cache for documentation
pub struct DocumentationCache {
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    config: CacheConfig,
}

impl DocumentationCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Arc::new(RwLock::new(LruCache::new(config.max_entries)));
        Self { cache, config }
    }

    /// Get a cached entry by key
    pub async fn get(&self, key: &str) -> Option<CacheEntry> {
        let mut cache = self.cache.write().await;
        cache.get(key).cloned()
    }

    /// Put an entry in the cache
    pub async fn put(&self, entry: CacheEntry) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.put(entry.key.clone(), entry);
        Ok(())
    }

    /// Check if an entry exists and is valid
    pub async fn is_valid(&self, key: &str) -> bool {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            entry.expires_at > Utc::now()
        } else {
            false
        }
    }

    /// Invalidate a specific cache entry
    pub async fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.pop(key);
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            len: cache.len(),
            max_capacity: cache.cap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub len: usize,
    pub max_capacity: usize,
}
