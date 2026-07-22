use serde::{Deserialize, Serialize};
use crate::identity::PeerId;

/// Permission for P2P operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    SendMessage,
    ReceiveMessage,
    Broadcast,
    DirectMessage,
    FileTransfer,
    TaskExecution,
    ResourceAccess,
    Admin,
}

/// Permission set for a peer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionSet {
    pub permissions: std::collections::HashSet<Permission>,
}

impl PermissionSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn revoke(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_any(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.has(p))
    }

    pub fn has_all(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.has(p))
    }
}

/// Access control entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    pub peer_id: PeerId,
    pub permissions: PermissionSet,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub granted_by: PeerId,
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

/// Access control list
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccessControlList {
    pub entries: Vec<AccessControlEntry>,
}

impl AccessControlList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, entry: AccessControlEntry) {
        self.entries.push(entry);
    }

    pub fn remove_entry(&mut self, peer_id: &PeerId) {
        self.entries.retain(|e| &e.peer_id != peer_id);
    }

    pub fn get_permissions(&self, peer_id: &PeerId) -> Option<&PermissionSet> {
        self.entries.iter()
            .find(|e| &e.peer_id == peer_id)
            .map(|e| &e.permissions)
    }

    pub fn has_permission(&self, peer_id: &PeerId, permission: &Permission) -> bool {
        self.get_permissions(peer_id)
            .map(|p| p.has(permission))
            .unwrap_or(false)
    }
}
