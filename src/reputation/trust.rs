use crate::reputation::types::*;

/// Trust level management and policies
pub struct TrustManager {
    config: ReputationConfig,
}

impl TrustManager {
    pub fn new(config: ReputationConfig) -> Self {
        Self { config }
    }

    /// Get trust level for a reputation score
    pub fn get_trust_level(&self, score: f64) -> TrustLevel {
        TrustLevel::from_score(score)
    }

    /// Check if a peer meets trust requirements
    pub fn meets_requirement(&self, reputation: &PeerReputation, required: TrustLevel) -> bool {
        reputation.trust_level >= required
    }

    /// Get recommended verification level for a peer
    pub fn recommended_verification(&self, reputation: &PeerReputation) -> VerificationLevel {
        match reputation.trust_level {
            TrustLevel::Untrusted => VerificationLevel::HumanApproval,
            TrustLevel::Low => VerificationLevel::IndependentVerifiers,
            TrustLevel::Medium => VerificationLevel::SampledChallenge,
            TrustLevel::High => VerificationLevel::HashAndSignature,
            TrustLevel::Verified => VerificationLevel::HashAndSignature,
        }
    }

    /// Calculate task acceptance probability based on reputation
    pub fn acceptance_probability(&self, reputation: &PeerReputation) -> f64 {
        match reputation.trust_level {
            TrustLevel::Untrusted => 0.1,
            TrustLevel::Low => 0.3,
            TrustLevel::Medium => 0.6,
            TrustLevel::High => 0.9,
            TrustLevel::Verified => 1.0,
        }
    }
}
