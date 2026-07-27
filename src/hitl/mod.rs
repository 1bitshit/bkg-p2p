//! Human-in-the-loop (HITL) approval state machine.
//!
//! Tools or actions that require explicit user confirmation
//! pass through this state machine before execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The lifecycle states of a HITL request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HitlStatus {
    /// Awaiting human review.
    Pending,
    /// Approved for execution.
    Approved,
    /// Rejected by human.
    Rejected,
    /// Escalated (no response within timeout).
    Escalated,
}

/// A request for human approval before an action is taken.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlRequest {
    pub id: String,
    pub run_id: Option<String>,
    pub tool_name: String,
    pub action: String,
    pub reason: String,
    pub risk_level: RiskLevel,
    pub parameters_preview: serde_json::Value,
    pub status: HitlStatus,
    pub created_at: String,
    pub decided_at: Option<String>,
    pub decided_by: Option<String>,
    pub error: Option<String>,
}

/// Risk level assigned to an action pending approval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

/// The HITL store (in-memory, backed optionally by redb).
#[derive(Clone)]
pub struct HitlStore {
    inner: Arc<RwLock<HashMap<String, HitlRequest>>>,
}

impl HitlStore {
    /// Create a new empty HITL store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a pending request and return its ID.
    pub async fn enqueue(&self, req: HitlRequest) -> String {
        let id = req.id.clone();
        self.inner.write().await.insert(id.clone(), req);
        id
    }

    /// Look up a request by ID.
    pub async fn get(&self, id: &str) -> Option<HitlRequest> {
        self.inner.read().await.get(id).cloned()
    }

    /// List all requests, optionally filtered by status.
    pub async fn list(&self, status: Option<HitlStatus>) -> Vec<HitlRequest> {
        let map = self.inner.read().await;
        if let Some(s) = status {
            map.values().filter(|r| r.status == s).cloned().collect()
        } else {
            map.values().cloned().collect()
        }
    }

    /// Approve a pending request.
    pub async fn approve(&self, id: &str, by: &str) -> Result<(), String> {
        let mut map = self.inner.write().await;
        let req = map
            .get_mut(id)
            .ok_or_else(|| format!("request {id} not found"))?;
        if req.status != HitlStatus::Pending {
            return Err(format!("request {id} is already {:?}", req.status));
        }
        req.status = HitlStatus::Approved;
        req.decided_at = Some(chrono::Utc::now().to_rfc3339());
        req.decided_by = Some(by.to_string());
        Ok(())
    }

    /// Reject a pending request.
    pub async fn reject(&self, id: &str, by: &str) -> Result<(), String> {
        let mut map = self.inner.write().await;
        let req = map
            .get_mut(id)
            .ok_or_else(|| format!("request {id} not found"))?;
        if req.status != HitlStatus::Pending {
            return Err(format!("request {id} is already {:?}", req.status));
        }
        req.status = HitlStatus::Rejected;
        req.decided_at = Some(chrono::Utc::now().to_rfc3339());
        req.decided_by = Some(by.to_string());
        Ok(())
    }

    /// Escalate a request that has timed out.
    pub async fn escalate(&self, id: &str) -> Result<(), String> {
        let mut map = self.inner.write().await;
        let req = map
            .get_mut(id)
            .ok_or_else(|| format!("request {id} not found"))?;
        req.status = HitlStatus::Escalated;
        req.decided_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Check whether a tool action can proceed (returns true for Approved or Low-risk with auto-approve).
    pub async fn is_approved(&self, id: &str) -> bool {
        match self.get(id).await {
            Some(r) => r.status == HitlStatus::Approved || r.risk_level == RiskLevel::Low,
            None => false,
        }
    }
}

impl Default for HitlStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> HitlRequest {
        HitlRequest {
            id: "hitl-1".to_string(),
            run_id: None,
            tool_name: "exec_shell".to_string(),
            action: "run".to_string(),
            reason: "Execute shell command".to_string(),
            risk_level: RiskLevel::High,
            parameters_preview: serde_json::json!({"cmd": "rm -rf /"}),
            status: HitlStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
            decided_at: None,
            decided_by: None,
            error: None,
        }
    }

    #[tokio::test]
    async fn test_enqueue_and_get() {
        let store = HitlStore::new();
        let req = sample_request();
        let id = store.enqueue(req).await;
        let fetched = store.get(&id).await.unwrap();
        assert_eq!(fetched.status, HitlStatus::Pending);
    }

    #[tokio::test]
    async fn test_approve_reject() {
        let store = HitlStore::new();
        let req = sample_request();
        let id = store.enqueue(req).await;

        store.approve(&id, "admin").await.unwrap();
        assert_eq!(store.get(&id).await.unwrap().status, HitlStatus::Approved);

        // re-enqueue another for reject
        let req2 = sample_request();
        let id2 = store.enqueue(req2).await;
        store.reject(&id2, "admin").await.unwrap();
        assert_eq!(store.get(&id2).await.unwrap().status, HitlStatus::Rejected);
    }

    #[tokio::test]
    async fn test_list_filtered() {
        let store = HitlStore::new();
        let req = sample_request();
        let id = store.enqueue(req).await;
        store.approve(&id, "admin").await.unwrap();

        let pending = store.list(Some(HitlStatus::Pending)).await;
        assert!(pending.is_empty());
        let approved = store.list(Some(HitlStatus::Approved)).await;
        assert_eq!(approved.len(), 1);
    }
}
