use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Peer reputation profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReputation {
    pub peer_id: String,
    pub overall_score: f64,
    pub capability_scores: HashMap<String, CapabilityScore>,
    pub success_rate: f64,
    pub abort_rate: f64,
    pub timeout_rate: f64,
    pub dispute_history: Vec<DisputeRecord>,
    pub proof_verification_rate: f64,
    pub avg_response_latency_ms: f64,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub total_interactions: u64,
    pub trust_level: TrustLevel,
    pub local_override: Option<f64>,
}

/// Capability-specific score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScore {
    pub capability: String,
    pub score: f64,
    pub interactions: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Dispute record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dispute_type: DisputeType,
    pub outcome: DisputeOutcome,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeType {
    TaskFailure,
    ProofRejection,
    PolicyViolation,
    Timeout,
    Fraud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeOutcome {
    ResolvedForPeer,
    ResolvedAgainstPeer,
    Pending,
    Dismissed,
}

/// Trust level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    Untrusted = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Verified = 4,
}

impl TrustLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.2 => TrustLevel::Untrusted,
            s if s < 0.4 => TrustLevel::Low,
            s if s < 0.6 => TrustLevel::Medium,
            s if s < 0.8 => TrustLevel::High,
            _ => TrustLevel::Verified,
        }
    }
}

/// Reputation event for tracking interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub peer_id: String,
    pub event_type: ReputationEventType,
    pub capability: Option<String>,
    pub weight: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationEventType {
    TaskCompleted,
    TaskFailed,
    TaskTimeout,
    ProofVerified,
    ProofRejected,
    DisputeResolved,
    HumanApproval,
    PolicyViolation,
}

/// Configuration for reputation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    pub enabled: bool,
    pub decay_half_life_days: f64,
    pub min_interactions_for_score: u64,
    pub capability_weights: HashMap<String, f64>,
    pub local_override_allowed: bool,
    pub persistence_path: Option<String>,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            decay_half_life_days: 30.0,
            min_interactions_for_score: 5,
            capability_weights: HashMap::new(),
            local_override_allowed: true,
            persistence_path: None,
        }
    }
}
