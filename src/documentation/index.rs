use crate::documentation::types::*;
use anyhow::Result;
use crate::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Documentation index for vector search
pub struct DocumentationIndex {
    vector_store: Option<Arc<VectorStore>>,
    config: IndexConfig,
    text_entries: Arc<RwLock<HashMap<String, (String, IndexMetadata)>>>,
}

impl DocumentationIndex {
    pub fn new(config: IndexConfig) -> Self {
        Self {
            vector_store: None,
            config,
            text_entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the vector store to use for indexing
    pub fn with_vector_store(mut self, store: Arc<VectorStore>) -> Self {
        self.vector_store = Some(store);
        self
    }

    /// Index documentation content into the store
    pub fn index_sync(&self, content: &str, metadata: IndexMetadata) -> Result<()> {
        let chunk_size = self.config.chunk_size;
        for (i, chunk) in content.as_bytes().chunks(chunk_size).enumerate() {
            let text = String::from_utf8_lossy(chunk).to_string();
            let id = format!("{}_chunk_{}", metadata.library_id.name, i);
            self.text_entries.write().insert(id, (text, metadata.clone()));
        }

        Ok(())
    }

    /// Index documentation content into the store (async wrapper)
    pub async fn index(&mut self, content: &str, metadata: IndexMetadata) -> Result<()> {
        self.index_sync(content, metadata)
    }

    /// Search the documentation index using text matching
    pub fn search_sync(&self, query: &str, limit: usize) -> Result<Vec<(IndexMetadata, f32)>> {
        let entries = self.text_entries.read();
        let query_lower = query.to_lowercase();
        let mut scored: Vec<(&str, &str, &IndexMetadata, f32)> = entries
            .iter()
            .filter_map(|(_id, (text, meta))| {
                let text_lower = text.to_lowercase();
                if text_lower.contains(&query_lower) {
                    let occurrences = text_lower.matches(&query_lower).count();
                    let score = occurrences as f32 / text.len().max(1) as f32;
                    Some((text.as_str(), query, meta, score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored
            .into_iter()
            .map(|(_text, _q, meta, score)| (meta.clone(), score))
            .collect())
    }

    /// Search the documentation index (async wrapper)
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<(IndexMetadata, f32)>> {
        self.search_sync(query, limit)
    }

    /// Delete entries for a specific library
    pub fn delete_library_sync(&self, library_id: &LibraryId) -> Result<()> {
        let mut entries = self.text_entries.write();
        entries.retain(|_id, (_text, meta)| meta.library_id != *library_id);
        Ok(())
    }

    /// Delete entries for a specific library (async wrapper)
    pub async fn delete_library(&self, library_id: &LibraryId) -> Result<()> {
        self.delete_library_sync(library_id)
    }
}
