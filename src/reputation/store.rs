use crate::db::Database;
use crate::reputation::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Persistent reputation store
pub struct ReputationStore {
    profiles: Arc<RwLock<HashMap<String, PeerReputation>>>,
    config: ReputationConfig,
    db: Option<Database>,
}

impl ReputationStore {
    pub fn new(config: ReputationConfig) -> Self {
        Self::new_with_db(config, None)
    }

    pub fn new_with_db(config: ReputationConfig, db: Option<Database>) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            config,
            db,
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
        let mut profiles = self.profiles.write().await;
        if let Some(profile) = profiles.get_mut(peer_id) {
            if self.config.local_override_allowed {
                profile.local_override = score;
                tracing::info!("Setting local override for {}: {:?}", peer_id, score);
            } else {
                tracing::warn!("Local overrides disabled in config, ignoring override for {}", peer_id);
            }
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

/// Persist all in-memory reputation profiles to the database.
///
/// If a database is attached, every profile is serialized with MessagePack
/// and written to the `reputation` table. Without a database this is a no-op.
pub async fn persist(&self) -> Result<()> {
    let db = match &self.db {
        Some(db) => db,
        None => return Ok(()),
    };

    let profiles = self.profiles.read().await;
    for (peer_id, profile) in profiles.iter() {
        db.store_reputation(peer_id, profile)?;
    }
    Ok(())
}

/// Load all reputation profiles from the database into memory.
///
/// Existing in-memory profiles are replaced by the persisted state.
/// Without a database this is a no-op.
pub async fn load(&self) -> Result<()> {
    let db = match &self.db {
        Some(db) => db,
        None => return Ok(()),
    };

    let ids = db.list_reputation_ids()?;
    let mut profiles = self.profiles.write().await;
    profiles.clear();

    for peer_id in ids {
        if let Some(profile) = db.get_reputation::<PeerReputation>(&peer_id)? {
            profiles.insert(peer_id, profile);
        }
    }
    Ok(())
}
}
