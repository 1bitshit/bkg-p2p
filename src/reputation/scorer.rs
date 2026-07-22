use crate::reputation::types::*;
use chrono::Utc;

/// Reputation scorer for calculating and updating scores
pub struct ReputationScorer {
    config: ReputationConfig,
}

impl ReputationScorer {
    pub fn new(config: ReputationConfig) -> Self {
        Self { config }
    }

    /// Calculate decayed score based on time since last update
    pub fn decay_score(&self, score: f64, last_updated: chrono::DateTime<chrono::Utc>) -> f64 {
        if !self.config.enabled {
            return score;
        }

        let now = Utc::now();
        let age_days = (now - last_updated).num_seconds() as f64 / 86400.0;

        // Exponential decay based on half-life
        let decay_factor = 0.5_f64.powf(age_days / self.config.decay_half_life_days);
        score * decay_factor
    }

    /// Calculate overall reputation score from events
    pub fn calculate_score(&self, events: &[ReputationEvent]) -> f64 {
        if events.is_empty() {
            return 0.5; // Default neutral score
        }

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for event in events {
            let weight = self.get_event_weight(&event.event_type);
            weighted_sum += weight * event.weight;
            total_weight += event.weight;
        }

        if total_weight == 0.0 {
            return 0.5;
        }

        // Normalize to 0.0-1.0 range
        let raw_score = weighted_sum / total_weight;
        (raw_score + 1.0) / 2.0
    }

    /// Get weight for an event type
    fn get_event_weight(&self, event_type: &ReputationEventType) -> f64 {
        match event_type {
            ReputationEventType::TaskCompleted => 1.0,
            ReputationEventType::TaskFailed => -1.0,
            ReputationEventType::TaskTimeout => -0.5,
            ReputationEventType::ProofVerified => 0.5,
            ReputationEventType::ProofRejected => -0.5,
            ReputationEventType::DisputeResolved => -1.0,
            ReputationEventType::HumanApproval => 0.3,
            ReputationEventType::PolicyViolation => -2.0,
        }
    }

    /// Determine if a peer is eligible for a task based on reputation
    pub fn is_eligible(&self, reputation: &PeerReputation, min_trust: TrustLevel) -> bool {
        if !self.config.enabled {
            return true;
        }

        reputation.trust_level >= min_trust
            && reputation.total_interactions >= self.config.min_interactions_for_score as u64
    }

    /// Calculate required verification level based on reputation
    pub fn required_verification(&self, reputation: &PeerReputation) -> VerificationLevel {
        match reputation.trust_level {
            TrustLevel::Untrusted => VerificationLevel::IndependentVerifiers,
            TrustLevel::Low => VerificationLevel::IndependentVerifiers,
            TrustLevel::Medium => VerificationLevel::SampledChallenge,
            TrustLevel::High => VerificationLevel::HashAndSignature,
            TrustLevel::Verified => VerificationLevel::HashAndSignature,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationLevel {
    HashAndSignature = 0,
    SampledChallenge = 1,
    IndependentVerifiers = 2,
    HumanApproval = 3,
}
