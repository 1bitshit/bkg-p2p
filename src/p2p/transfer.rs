use serde::{Deserialize, Serialize};
use crate::identity::PeerId;

/// File transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferRequest {
    pub id: String,
    pub sender: PeerId,
    pub receiver: PeerId,
    pub filename: String,
    pub size: u64,
    pub mime_type: String,
    pub chunk_size: u32,
    pub total_chunks: u32,
    pub checksum: Option<String>,
}

impl FileTransferRequest {
    pub fn new(sender: PeerId, receiver: PeerId, filename: String, size: u64) -> Self {
        let chunk_size = 65536; // 64KB chunks
        let total_chunks = ((size as f64) / (chunk_size as f64)).ceil() as u32;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender,
            receiver,
            filename,
            size,
            mime_type: "application/octet-stream".to_string(),
            chunk_size,
            total_chunks,
            checksum: None,
        }
    }
}

/// File transfer chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferChunk {
    pub transfer_id: String,
    pub chunk_index: u32,
    pub data: Vec<u8>,
    pub checksum: Option<String>,
}

/// File transfer status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferStatus {
    pub transfer_id: String,
    pub state: TransferState,
    pub bytes_transferred: u64,
    pub chunks_received: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferState {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}
