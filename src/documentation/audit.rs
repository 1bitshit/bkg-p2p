use crate::documentation::types::*;
use chrono::Utc;

/// Audit log for documentation operations
pub struct DocumentationAudit {
    entries: Vec<AuditEntry>,
}

impl DocumentationAudit {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Log a documentation access event
    pub fn log_access(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    /// Get recent audit entries
    pub fn recent(&self, limit: usize) -> &[AuditEntry] {
        let start = self.entries.len().saturating_sub(limit);
        &self.entries[start..]
    }

    /// Clear old entries
    pub fn cleanup(&mut self, older_than: chrono::DateTime<Utc>) {
        self.entries.retain(|e| e.timestamp > older_than);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<Utc>,
    pub operation: AuditOperation,
    pub library_id: Option<LibraryId>,
    pub provider: Option<String>,
    pub peer_id: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOperation {
    Resolve,
    Fetch,
    Index,
    Search,
    CacheHit,
    CacheMiss,
    CacheWrite,
    ProviderHealthCheck,
}
