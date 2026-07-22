use crate::documentation::types::*;
use anyhow::Result;

/// Library resolver for documentation lookups
pub struct LibraryResolver {
    providers: Vec<Box<dyn DocumentationProvider>>,
}

impl LibraryResolver {
    pub fn new(providers: Vec<Box<dyn DocumentationProvider>>) -> Self {
        Self { providers }
    }

    /// Resolve a library query across all enabled providers
    pub async fn resolve(&self, request: ResolveLibraryRequest) -> Result<Vec<LibraryMatch>> {
        let mut all_matches = Vec::new();

        for provider in &self.providers {
            match provider.resolve_library(request.clone()).await {
                Ok(matches) => all_matches.extend(matches),
                Err(e) => tracing::warn!("Provider failed to resolve library: {}", e),
            }
        }

        // Sort by confidence descending
        all_matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate by library_id
        all_matches.dedup_by(|a, b| a.library_id == b.library_id);

        Ok(all_matches)
    }

    /// Find the best match for a library query
    pub async fn resolve_best(&self, request: ResolveLibraryRequest) -> Result<Option<LibraryMatch>> {
        let matches = self.resolve(request).await?;
        Ok(matches.into_iter().next())
    }
}
