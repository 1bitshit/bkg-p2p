use serde::{Deserialize, Serialize};
use crate::identity::PeerId;
use crate::p2p::envelope::P2PEnvelope;

/// Peer invitation for direct connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInvite {
    pub id: String,
    pub inviter: PeerId,
    pub invitee: Option<PeerId>,
    pub topic: String,
    pub payload: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: u32,
    pub signature: Option<Vec<u8>>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl PeerInvite {
    pub fn new(inviter: PeerId, topic: String, payload: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            inviter,
            invitee: None,
            topic,
            payload,
            timestamp: chrono::Utc::now(),
            ttl: 64,
            signature: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn to_envelope(&self) -> P2PEnvelope {
        P2PEnvelope {
            id: self.id.clone(),
            source: self.inviter.clone(),
            target: self.invitee.clone(),
            topic: self.topic.clone(),
            payload: self.payload.clone(),
            timestamp: self.timestamp,
            ttl: self.ttl,
            signature: self.signature.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Invite response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteResponse {
    pub invite_id: String,
    pub accepted: bool,
    pub responder: PeerId,
    pub error: Option<String>,
}
