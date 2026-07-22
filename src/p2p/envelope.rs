use serde::{Deserialize, Serialize};
use crate::identity::PeerId;

/// Standardized P2P message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PEnvelope {
    pub id: String,
    pub source: PeerId,
    pub target: Option<PeerId>,
    pub topic: String,
    pub payload: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: u32,
    pub signature: Option<Vec<u8>>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl P2PEnvelope {
    pub fn new(source: PeerId, topic: String, payload: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            target: None,
            topic,
            payload,
            timestamp: chrono::Utc::now(),
            ttl: 64,
            signature: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_target(mut self, target: PeerId) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Decrement TTL and check if message is still valid
    pub fn decrement_ttl(&mut self) -> bool {
        if self.ttl == 0 {
            return false;
        }
        self.ttl -= 1;
        true
    }
}
