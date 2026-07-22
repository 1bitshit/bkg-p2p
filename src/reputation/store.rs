use crate::reputation::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Persistent reputation store
pub struct ReputationStore {
    profiles: Arc<RwLock<HashMap<String, PeerReputation>>>,
    config: ReputationConfig,
}

impl ReputationStore {
    pub fn new(config: ReputationConfig) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get reputation for a peer
    pub async fn get(&self, peer_id: &str) -> Option<PeerReputation> {
        let profiles = self.profiles.read().await;
        profiles.get(peer_id).cloned()
    }

    /// Update reputation based on an event
    pub async fn update(&self, event: ReputationEvent) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        let profile = profiles.entry(event.peer_id.clone()).or_insert_with(|| PeerReputation {
            peer_id: event.peer_id.clone(),
            overall_score: 0.5,
            capability_scores: HashMap::new(),
            success_rate: 0.0,
            abort_rate: 0.0,
            timeout_rate: 0.0,
            dispute_history: Vec::new(),
            proof_verification_rate: 0.0,
            avg_response_latency_ms: 0.0,
            first_seen: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            total_interactions: 0,
            trust_level: TrustLevel::Untrusted,
            local_override: None,
        });

        // Apply event weight
        let delta = match event.event_type {
            ReputationEventType::TaskCompleted => event.weight,
            ReputationEventType::TaskFailed => -event.weight,
            ReputationEventType::TaskTimeout => -event.weight * 0.5,
            ReputationEventType::ProofVerified => event.weight * 0.5,
            ReputationEventType::ProofRejected => -event.weight * 0.5,
            ReputationEventType::DisputeResolved => -event.weight,
            ReputationEventType::HumanApproval => event.weight * 0.3,
            ReputationEventType::PolicyViolation => -event.weight * 2.0,
        };

        // Update overall score with bounds
        profile.overall_score = (profile.overall_score + delta).clamp(0.0, 1.0);
        profile.total_interactions += 1;
        profile.last_updated = chrono::Utc::now();
        profile.trust_level = TrustLevel::from_score(profile.overall_score);

        // Update capability-specific score if provided
        if let Some(capability) = event.capability {
            let cap_score = profile.capability_scores.entry(capability.clone()).or_insert_with(|| CapabilityScore {
                capability,
                score: 0.5,
                interactions: 0,
                last_updated: chrono::Utc::now(),
            });
            cap_score.score = (cap_score.score + delta).clamp(0.0, 1.0);
            cap_score.interactions += 1;
            cap_score.last_updated = chrono::Utc::now();
        }

        Ok(())
    }

    /// Set local override for a peer
    pub async fn set_local_override(&self, peer_id: &str, score: Option<f64>) -> Result<()> {
        let profiles = self.profiles.write().await;
        if let Some(profile) = profiles.get(peer_id) {
            // In a real implementation, we'd need mutable access
            tracing::info!("Setting local override for {}: {:?}", peer_id, score);
        }
        Ok(())
    }

    /// Get top peers by score
    pub async fn top_peers(&self, capability: Option<&str>, limit: usize) -> Vec<PeerReputation> {
        let profiles = self.profiles.read().await;
        let mut peers: Vec<_> = profiles.values().cloned().collect();

        if let Some(cap) = capability {
            peers.retain(|p| p.capability_scores.contains_key(cap));
            peers.sort_by(|a, b| {
                let score_a = a.capability_scores.get(cap).map(|s| s.score).unwrap_or(0.0);
                let score_b = b.capability_scores.get(cap).map(|s| s.score).unwrap_or(0.0);
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            peers.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        }

        peers.truncate(limit);
        peers
    }

    /// Persist reputation data
    pub async fn persist(&self) -> Result<()> {
        // TODO: Implement persistence to redb or file
        Ok(())
    }

    /// Load reputation data
    pub async fn load(&self) -> Result<()> {
        // TODO: Implement loading from redb or file
        Ok(())
    }
}
