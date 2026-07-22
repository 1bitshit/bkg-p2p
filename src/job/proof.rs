use serde::{Deserialize, Serialize};
use crate::identity::PeerId;

/// Proof of work or task completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProof {
    pub id: String,
    pub task_id: String,
    pub prover: PeerId,
    pub proof_type: ProofType,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub verified: bool,
    pub verification_result: Option<VerificationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    HashCommitment,
    MerkleProof,
    ZKProof,
    ExecutionTrace,
    OutputHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verified: bool,
    pub verifier: PeerId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub confidence: f64,
    pub notes: Option<String>,
}

/// Proof verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationRequest {
    pub proof_id: String,
    pub verifier: PeerId,
    pub required_confidence: f64,
}
