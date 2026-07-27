//! Tool manifest GossipSub advertisement for cross-peer tool discovery.

use serde::{Deserialize, Serialize};

/// GossipSub topic for tool manifests.
pub const TOOL_MANIFEST_TOPIC: &str = "bkg-p2p/tools/v1";

/// A tool that a peer advertises for remote execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    /// Unique tool identifier (e.g. `google-search`, `github-issues`).
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// Tool category (e.g. `web`, `github`, `file`, `messaging`).
    pub category: String,
    /// JSON Schema for the tool's input parameters.
    pub parameters_schema: serde_json::Value,
    /// Whether this tool requires local execution (true) or can be run remotely (false).
    pub local_only: bool,
    /// Capability tags for access control (e.g. `web.egress`, `storage.read`).
    pub capabilities: Vec<String>,
    /// Peer ID of the tool provider.
    pub provider_peer_id: String,
    /// Unix timestamp when this manifest was created.
    pub created_at: i64,
    /// Optional expiry timestamp (0 = no expiry).
    pub expires_at: i64,
    /// Ed25519 signature of the manifest body.
    pub signature: Option<String>,
}

impl ToolManifest {
    /// Create a new unsigned tool manifest.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        parameters_schema: serde_json::Value,
        local_only: bool,
        capabilities: Vec<String>,
        provider_peer_id: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: category.into(),
            parameters_schema,
            local_only,
            capabilities,
            provider_peer_id: provider_peer_id.into(),
            created_at: chrono::Utc::now().timestamp(),
            expires_at: 0,
            signature: None,
        }
    }

    /// Check if this manifest is expired (0 means no expiry).
    pub fn is_expired(&self, max_age_secs: u64) -> bool {
        if self.expires_at > 0 {
            let now = chrono::Utc::now().timestamp();
            now > self.expires_at
        } else if max_age_secs > 0 {
            let now = chrono::Utc::now().timestamp();
            now - self.created_at > max_age_secs as i64
        } else {
            false
        }
    }

    /// Sign the manifest with a provided signer function.
    pub fn sign<F>(&mut self, signer: F)
    where
        F: FnOnce(&[u8]) -> String,
    {
        let bytes = self.signing_bytes();
        self.signature = Some(signer(&bytes));
    }

    /// Returns canonical serialization bytes for signing/verification.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let unsigned = serde_json::json!({
            "name": self.name,
            "description": self.description,
            "category": self.category,
            "parameters_schema": self.parameters_schema,
            "local_only": self.local_only,
            "capabilities": self.capabilities,
            "provider_peer_id": self.provider_peer_id,
            "created_at": self.created_at,
            "expires_at": self.expires_at,
        });
        serde_json::to_vec(&unsigned).unwrap_or_default()
    }
}

/// A set of tool manifests advertised over the network.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolManifestAdvert {
    /// Peer ID of the advertiser.
    pub peer_id: String,
    /// List of tools offered.
    pub tools: Vec<ToolManifest>,
}

impl ToolManifestAdvert {
    /// Create a new empty tool manifest advertisement.
    pub fn new(peer_id: impl Into<String>) -> Self {
        Self {
            peer_id: peer_id.into(),
            tools: Vec::new(),
        }
    }

    /// Add a tool manifest to the advertisement.
    pub fn add_tool(&mut self, tool: ToolManifest) {
        self.tools.push(tool);
    }

    /// Filter to only non-local tools (suitable for remote execution).
    pub fn remote_tools(&self) -> Vec<&ToolManifest> {
        self.tools.iter().filter(|t| !t.local_only).collect()
    }

    /// Filter to only tools matching a category.
    pub fn tools_by_category(&self, category: &str) -> Vec<&ToolManifest> {
        self.tools
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_manifest_creation() {
        let manifest = ToolManifest::new(
            "web-search",
            "Search the web for information",
            "web",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }),
            false,
            vec!["web.egress".to_string()],
            "peer-123",
        );
        assert_eq!(manifest.name, "web-search");
        assert!(!manifest.local_only);
        assert_eq!(manifest.capabilities, vec!["web.egress"]);
        assert!(manifest.signature.is_none());
    }

    #[test]
    fn test_tool_manifest_serialization() {
        let manifest = ToolManifest::new(
            "test-tool",
            "A test tool",
            "test",
            serde_json::json!({"type": "object"}),
            true,
            vec![],
            "peer-abc",
        );
        let bytes = rmp_serde::to_vec(&manifest).unwrap();
        let decoded: ToolManifest = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(decoded.name, "test-tool");
    }

    #[test]
    fn test_tool_advert() {
        let mut advert = ToolManifestAdvert::new("peer-xyz");
        advert.add_tool(ToolManifest::new(
            "tool-1",
            "Desc 1",
            "cat-a",
            serde_json::json!({}),
            false,
            vec![],
            "peer-xyz",
        ));
        advert.add_tool(ToolManifest::new(
            "tool-2",
            "Desc 2",
            "cat-b",
            serde_json::json!({}),
            true,
            vec![],
            "peer-xyz",
        ));
        assert_eq!(advert.tools.len(), 2);
        assert_eq!(advert.remote_tools().len(), 1);
        assert_eq!(advert.tools_by_category("cat-a").len(), 1);
    }

    #[test]
    fn test_manifest_not_expired_by_default() {
        let manifest = ToolManifest::new(
            "fresh",
            "Fresh tool",
            "cat",
            serde_json::json!({}),
            false,
            vec![],
            "peer",
        );
        assert!(!manifest.is_expired(300));
    }
}