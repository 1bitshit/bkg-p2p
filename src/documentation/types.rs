use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Library identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LibraryId {
    pub name: String,
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Ecosystem {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Other(String),
}

/// Request to resolve a library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveLibraryRequest {
    pub query: String,
    pub version: Option<String>,
    pub language: Option<String>,
    pub context: Option<String>,
}

/// Match result from library resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMatch {
    pub library_id: LibraryId,
    pub versions: Vec<String>,
    pub sources: Vec<DocumentationSource>,
    pub confidence: f32,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStatus {
    Hit,
    Miss,
    Stale,
}

/// Documentation source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSource {
    pub source_type: SourceType,
    pub url: Option<String>,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Local,
    Workspace,
    Rustdoc,
    CratesIo,
    Context7,
    Other(String),
}

/// Request for documentation content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationRequest {
    pub library_id: LibraryId,
    pub query: String,
    pub version: Option<String>,
    pub max_size: Option<usize>,
    pub preferred_sources: Vec<DocumentationSource>,
}

/// Result from fetching documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationResult {
    pub sections: Vec<DocumentationSection>,
    pub sources: Vec<DocumentationSource>,
    pub version: Option<String>,
    pub fetch_time_ms: u64,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSection {
    pub title: String,
    pub content: String,
    pub source: DocumentationSource,
    pub section_path: Vec<String>,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub available: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Cache entry for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub library_id: LibraryId,
    pub content: DocumentationResult,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_hash: String,
}

/// Index metadata for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub library_id: LibraryId,
    pub source_url: Option<String>,
    pub local_origin: Option<String>,
    pub section: String,
    pub language: Option<String>,
    pub content_hash: String,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}

/// Configuration for documentation subsystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub enabled: bool,
    pub providers: Vec<ProviderConfig>,
    pub cache: CacheConfig,
    pub index: IndexConfig,
    pub max_response_size: usize,
    pub request_timeout_secs: u64,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: SourceType,
    pub enabled: bool,
    pub priority: u32,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub max_size_bytes: usize,
    pub default_ttl_secs: u64,
    pub lru_eviction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub collection_name: String,
    pub embedding_model: Option<String>,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub concurrent_requests: u32,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            providers: vec![
                ProviderConfig {
                    provider_type: SourceType::Local,
                    enabled: true,
                    priority: 1,
                    config: HashMap::new(),
                },
                ProviderConfig {
                    provider_type: SourceType::Workspace,
                    enabled: true,
                    priority: 2,
                    config: HashMap::new(),
                },
                ProviderConfig {
                    provider_type: SourceType::Rustdoc,
                    enabled: true,
                    priority: 3,
                    config: HashMap::new(),
                },
                ProviderConfig {
                    provider_type: SourceType::CratesIo,
                    enabled: true,
                    priority: 4,
                    config: HashMap::new(),
                },
            ],
            cache: CacheConfig {
                max_entries: 10000,
                max_size_bytes: 500 * 1024 * 1024, // 500MB
                default_ttl_secs: 86400, // 24 hours
                lru_eviction: true,
            },
            index: IndexConfig {
                collection_name: "documentation".to_string(),
                embedding_model: None,
                chunk_size: 512,
                chunk_overlap: 64,
            },
            max_response_size: 2 * 1024 * 1024, // 2MB
            request_timeout_secs: 30,
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                concurrent_requests: 10,
            },
        }
    }
}
