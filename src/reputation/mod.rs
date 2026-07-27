//! BKG P2P Reputation System
//!
//! Persistent peer reputation with capability-specific scores,
//! decay, and trust levels.

pub mod scorer;
pub mod store;
pub mod trust;
pub mod types;

pub use scorer::{ReputationScorer, VerificationLevel};
pub use store::ReputationStore;
pub use trust::TrustManager;
pub use types::*;
