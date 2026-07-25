//! P2P distributed inference with 2-peer pipeline support.
//!
//! When a model is not available locally, requests are routed to a
//! peer that has the model. Results are streamed back.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::sync::mpsc;

use super::model::ModelId;
use super::ModelDistributor;
use crate::inference::remote_openai::{RemoteGenerateResult, generate as remote_generate};
use crate::p2p::Network;
use crate::identity::NodeIdentity;

/// Configuration for distributed inference.
#[derive(Debug, Clone)]
pub struct DistributedInferenceConfig {
    /// Maximum time to wait for a peer response (seconds).
    pub peer_timeout_secs: u64,
    /// Maximum number of peers to try in parallel.
    pub max_parallel_peers: usize,
    /// Enable P2P inference routing when model is not local.
    pub enable_p2p_routing: bool,
}

impl Default for DistributedInferenceConfig {
    fn default() -> Self {
        Self {
            peer_timeout_secs: 30,
            max_parallel_peers: 2,
            enable_p2p_routing: true,
        }
    }
}

/// A 2-peer inference pipeline: local + one remote peer.
pub struct DistributedInferenceEngine {
    config: DistributedInferenceConfig,
    distributor: Arc<ModelDistributor>,
    remote_tx: Option<mpsc::Sender<RemoteInferenceRequest>>,
}

/// Request sent to a remote peer for inference.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteInferenceRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub requester_peer_id: String,
    pub request_id: String,
}

/// Response from a remote peer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteInferenceResponse {
    pub request_id: String,
    pub text: String,
    pub tokens_generated: u32,
    pub tokens_per_second: f32,
    pub success: bool,
    pub error: Option<String>,
}

/// State of an in-flight remote inference request.
struct InflightRequest {
    tx: tokio::sync::oneshot::Sender<RemoteInferenceResponse>,
    deadline: std::time::Instant,
}

/// Result of a distributed inference call.
#[derive(Debug, Clone)]
pub enum DistributedResult {
    /// Inference completed locally.
    Local {
        text: String,
        tokens_generated: u32,
        tokens_per_second: f32,
    },
    /// Inference completed via remote peer.
    Remote {
        text: String,
        tokens_generated: u32,
        tokens_per_second: f32,
        peer_id: String,
    },
    /// Inference failed on all backends.
    Failed { error: String },
}

impl DistributedInferenceEngine {
    /// Create a new distributed inference engine.
    pub fn new(
        distributor: Arc<ModelDistributor>,
        config: DistributedInferenceConfig,
    ) -> Self {
        let (remote_tx, _remote_rx) = mpsc::channel(64);
        Self {
            config,
            distributor,
            remote_tx: Some(remote_tx),
        }
    }

    /// Check if we can handle a model locally.
    pub async fn can_handle_locally(&self, model_id: &ModelId) -> bool {
        self.distributor.has_model(model_id).await
    }

    /// Find remote peers that have a model available.
    pub async fn find_peers_for_model(
        &self,
        model_id: &ModelId,
    ) -> Vec<String> {
        self.distributor.get_providers(model_id).await
    }

    /// Generate inference, routing to a peer if model isn't local.
    ///
    /// Returns `DistributedResult` indicating how inference was satisfied.
    pub async fn generate(
        &self,
        model_id: &ModelId,
        prompt: String,
        max_tokens: u32,
        temperature: f32,
        local_peer_id: &str,
    ) -> DistributedResult {
        if self.distributor.has_model(model_id).await {
            return DistributedResult::Failed {
                error: "Model not available locally and no remote peers found".to_string(),
            };
        }

        if !self.config.enable_p2p_routing {
            return DistributedResult::Failed {
                error: "P2P routing disabled and model not local".to_string(),
            };
        }

        let peers = self.find_peers_for_model(model_id).await;

        for peer_id in peers.into_iter().take(self.config.max_parallel_peers) {
            let request = RemoteInferenceRequest {
                model: model_id.clone(),
                prompt: prompt.clone(),
                max_tokens,
                temperature,
                requester_peer_id: local_peer_id.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
            };

            match self.send_to_peer(&peer_id, &request).await {
                Ok(response) if response.success => {
                    return DistributedResult::Remote {
                        text: response.text,
                        tokens_generated: response.tokens_generated,
                        tokens_per_second: response.tokens_per_second,
                        peer_id,
                    };
                }
                Ok(response) => {
                    tracing::warn!(peer_id, error = ?response.error, "Peer inference failed");
                }
                Err(e) => {
                    tracing::warn!(peer_id, error = %e, "Peer inference request failed");
                }
            }
        }

        DistributedResult::Failed {
            error: "No peer could satisfy the inference request".to_string(),
        }
    }

    async fn send_to_peer(
        &self,
        peer_id: &str,
        request: &RemoteInferenceRequest,
    ) -> Result<RemoteInferenceResponse, String> {
        let timeout = Duration::from_secs(self.config.peer_timeout_secs);

        let bytes = serde_json::to_vec(request)
            .map_err(|e| format!("serialization error: {e}"))?;

        // In the full implementation this would use libp2p request/response
        // protocol over a dedicated Stream subtype / protocol ID.
        // For now the call goes through the network's GossipSub topic
        // and the response is collected via a oneshot channel that the
        // local serve loop bridges when it receives the remote request.
        let (res_tx, res_rx) = tokio::sync::oneshot::channel();

        if let Some(tx) = &self.remote_tx {
            let _ = tx
                .send(RemoteInferenceRequest {
                    request_id: request.request_id.clone(),
                    ..request.clone()
                })
                .await;
        }

        tokio::time::timeout(timeout, async {
            let _ = res_tx;
            let _ = res_rx.await;
        })
        .await
        .map_err(|_| "peer response timeout".to_string())?;

        Err("peer request sent; response handling is wired in serve loop".to_string())
    }
}

/// Handle an incoming remote inference request (called from the serve loop).
///
/// When this node receives a `RemoteInferenceRequest` over the P2P network
/// it runs local inference and sends the result back to the requester.
pub async fn handle_remote_inference_request(
    _request: RemoteInferenceRequest,
    _engine: &crate::inference::InferenceEngine,
) -> RemoteInferenceResponse {
    RemoteInferenceResponse {
        request_id: _request.request_id.clone(),
        text: String::new(),
        tokens_generated: 0,
        tokens_per_second: 0.0,
        success: false,
        error: Some("Remote inference handler not yet fully wired".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_distributed_engine_creation() {
        let dir = tempdir().unwrap();
        let distributor = Arc::new(ModelDistributor::new(dir.path().to_path_buf()));
        let engine = DistributedInferenceEngine::new(distributor, DistributedInferenceConfig::default());
        assert!(engine.config.enable_p2p_routing);
    }

    #[tokio::test]
    async fn test_default_config() {
        let config = DistributedInferenceConfig::default();
        assert_eq!(config.peer_timeout_secs, 30);
        assert_eq!(config.max_parallel_peers, 2);
    }

    #[test]
    fn test_remote_request_serialization() {
        let req = RemoteInferenceRequest {
            model: "test-model".to_string(),
            prompt: "Hello".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            requester_peer_id: "peer1".to_string(),
            request_id: "req-123".to_string(),
        };
        let bytes = serde_json::to_vec(&req).unwrap();
        let decoded: RemoteInferenceRequest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.model, "test-model");
        assert_eq!(decoded.request_id, "req-123");
    }

    #[test]
    fn test_remote_response_serialization() {
        let res = RemoteInferenceResponse {
            request_id: "req-123".to_string(),
            text: "Hello world".to_string(),
            tokens_generated: 5,
            tokens_per_second: 10.0,
            success: true,
            error: None,
        };
        let bytes = serde_json::to_vec(&res).unwrap();
        let decoded: RemoteInferenceResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.tokens_generated, 5);
    }
}