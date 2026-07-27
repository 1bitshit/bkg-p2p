use crate::documentation::audit::{AuditEntry, AuditOperation};
use crate::documentation::types::*;
use crate::documentation::{DocumentationAudit, DocumentationProvider, LibraryResolver};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Documentation Agent Capability
pub struct DocumentationAgent {
    resolver: LibraryResolver,
    cache: crate::documentation::cache::DocumentationCache,
    index: crate::documentation::index::DocumentationIndex,
    audit: Arc<Mutex<DocumentationAudit>>,
    config: DocumentationConfig,
}

impl DocumentationAgent {
    pub fn new(config: DocumentationConfig) -> Self {
        let providers: Vec<Box<dyn DocumentationProvider>> = vec![
            Box::new(crate::documentation::provider::LocalDocsProvider {
                root_path: std::path::PathBuf::from("./docs"),
            }),
            Box::new(crate::documentation::provider::WorkspaceDocsProvider {
                workspace_path: std::path::PathBuf::from("."),
            }),
        ];

        let resolver = LibraryResolver::new(providers);
        let cache = crate::documentation::cache::DocumentationCache::new(config.cache.clone());
        let index = crate::documentation::index::DocumentationIndex::new(config.index.clone());

        Self {
            resolver,
            cache,
            index,
            audit: Arc::new(Mutex::new(DocumentationAudit::new())),
            config,
        }
    }

    /// Resolve a library query
    pub async fn resolve_library(
        &self,
        request: ResolveLibraryRequest,
    ) -> Result<Vec<LibraryMatch>> {
        let result = self.resolver.resolve(request.clone()).await?;
        Ok(result)
    }

    /// Fetch documentation for a library
    pub async fn fetch_docs(&self, request: DocumentationRequest) -> Result<DocumentationResult> {
        let mut audit = self.audit.lock().await;
        let cache_key = format!(
            "{}:{}",
            request.library_id.name,
            request.library_id.version.as_deref().unwrap_or("latest")
        );

        if self.cache.is_valid(&cache_key).await {
            if let Some(entry) = self.cache.get(&cache_key).await {
                audit.log_access(AuditEntry {
                    timestamp: Utc::now(),
                    operation: AuditOperation::CacheHit,
                    library_id: Some(request.library_id.clone()),
                    provider: None,
                    peer_id: None,
                    success: true,
                    error: None,
                    metadata: std::collections::HashMap::new(),
                });
                return Ok(entry.content);
            }
        }

        let mut last_error = None;
        for provider in &self.resolver.providers {
            match provider.fetch_docs(request.clone()).await {
                Ok(result) => {
                    let entry = CacheEntry {
                        key: cache_key.clone(),
                        library_id: request.library_id.clone(),
                        content: result.clone(),
                        created_at: Utc::now(),
                        expires_at: Utc::now()
                            + chrono::Duration::seconds(self.config.cache.default_ttl_secs as i64),
                        etag: None,
                        last_modified: None,
                        content_hash: result.content_hash.clone(),
                    };
                    let _ = self.cache.put(entry).await;
                    audit.log_access(AuditEntry {
                        timestamp: Utc::now(),
                        operation: AuditOperation::Fetch,
                        library_id: Some(request.library_id.clone()),
                        provider: Some(cache_key.clone()),
                        peer_id: None,
                        success: true,
                        error: None,
                        metadata: std::collections::HashMap::new(),
                    });
                    return Ok(result);
                }
                Err(e) => {
                    audit.log_access(AuditEntry {
                        timestamp: Utc::now(),
                        operation: AuditOperation::Fetch,
                        library_id: Some(request.library_id.clone()),
                        provider: None,
                        peer_id: None,
                        success: false,
                        error: Some(e.to_string()),
                        metadata: std::collections::HashMap::new(),
                    });
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("No documentation provider available")))
    }

    /// Search local documentation index
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<(IndexMetadata, f32)>> {
        self.index.search(query, limit).await
    }

    /// Index documentation content
    pub async fn index(&mut self, content: &str, metadata: IndexMetadata) -> Result<()> {
        self.index.index(content, metadata).await
    }

    /// Get provider health status
    pub async fn provider_health(&self) -> Vec<(String, ProviderHealth)> {
        let mut results = Vec::new();
        for (i, provider) in self.resolver.providers.iter().enumerate() {
            let name = format!("provider_{}", i);
            let health = provider.health().await;
            results.push((name, health));
        }
        results
    }
}
