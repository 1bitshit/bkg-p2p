//! Payment verification for HTTP 402 proxy.
//!
//! Implements real cryptographic verification for three payment methods:
//! - Direct: ed25519 signature over `proof_id:amount`
//! - Channel: ed25519 signature over `(channel_id, nonce, amount)` with nonce replay protection
//! - Prepaid: API key lookup with balance deduction

use axum::http::HeaderMap;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Payment method for proxy requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    /// Direct per-request payment with signed proof
    Direct {
        /// Transaction or proof ID
        proof_id: String,
        /// Amount paid in μBKG
        amount: u64,
        /// Signature from payer
        signature: String,
    },
    /// Payment channel micropayment
    Channel {
        /// Channel ID
        channel_id: String,
        /// State nonce
        nonce: u64,
        /// Amount for this request
        amount: u64,
        /// Signed state update
        signature: String,
    },
    /// Prepaid balance on account
    Prepaid {
        /// Account identifier
        account_id: String,
        /// API key or auth token
        api_key: String,
    },
}

/// Payment proof extracted from request headers.
#[derive(Debug, Clone)]
pub struct PaymentProof {
    /// Payment method
    pub method: PaymentMethod,
    /// Timestamp of payment
    pub timestamp: u64,
}

impl PaymentProof {
    /// Extract payment proof from HTTP headers.
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        // Check for different payment header formats

        // X-Payment-Proof: direct:<proof_id>:<amount>:<signature>
        if let Some(proof) = headers.get("X-Payment-Proof") {
            if let Ok(proof_str) = proof.to_str() {
                return Self::parse_proof(proof_str);
            }
        }

        // X-Channel-Payment: <channel_id>:<nonce>:<amount>:<signature>
        if let Some(channel_payment) = headers.get("X-Channel-Payment") {
            if let Ok(payment_str) = channel_payment.to_str() {
                return Self::parse_channel_payment(payment_str);
            }
        }

        // Authorization: Bearer <api_key> for prepaid accounts
        if let Some(auth) = headers.get("Authorization") {
            if let Ok(auth_str) = auth.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let api_key = auth_str.trim_start_matches("Bearer ").to_string();
                    return Some(Self {
                        method: PaymentMethod::Prepaid {
                            account_id: String::new(), // Will be looked up
                            api_key,
                        },
                        timestamp: chrono::Utc::now().timestamp() as u64,
                    });
                }
            }
        }

        None
    }

    /// Parse a direct payment proof.
    fn parse_proof(proof: &str) -> Option<Self> {
        let parts: Vec<&str> = proof.split(':').collect();

        if parts.len() >= 4 && parts[0] == "direct" {
            let proof_id = parts[1].to_string();
            let amount = parts[2].parse().ok()?;
            let signature = parts[3].to_string();

            return Some(Self {
                method: PaymentMethod::Direct {
                    proof_id,
                    amount,
                    signature,
                },
                timestamp: chrono::Utc::now().timestamp() as u64,
            });
        }

        None
    }

    /// Parse a channel payment proof.
    fn parse_channel_payment(payment: &str) -> Option<Self> {
        let parts: Vec<&str> = payment.split(':').collect();

        if parts.len() >= 4 {
            let channel_id = parts[0].to_string();
            let nonce = parts[1].parse().ok()?;
            let amount = parts[2].parse().ok()?;
            let signature = parts[3].to_string();

            return Some(Self {
                method: PaymentMethod::Channel {
                    channel_id,
                    nonce,
                    amount,
                    signature,
                },
                timestamp: chrono::Utc::now().timestamp() as u64,
            });
        }

        None
    }

    /// Verify the payment proof against the required amount.
    pub fn verify(&self, required_amount: u64) -> Result<bool, String> {
        match &self.method {
            PaymentMethod::Direct {
                amount,
                signature,
                proof_id,
            } => {
                // Check amount
                if *amount < required_amount {
                    return Ok(false);
                }

                // TODO: Verify signature against known public key
                // For now, just check that signature is present
                if signature.is_empty() {
                    return Err("Empty signature".into());
                }

                // TODO: Check proof_id hasn't been used before (replay protection)
                if proof_id.is_empty() {
                    return Err("Empty proof ID".into());
                }

                Ok(true)
            }

            PaymentMethod::Channel {
                amount,
                signature,
                channel_id,
                nonce: _,
            } => {
                // Check amount
                if *amount < required_amount {
                    return Ok(false);
                }

                // TODO: Verify channel exists and has balance
                // TODO: Verify signature is valid state update
                // TODO: Verify nonce is incremented

                if signature.is_empty() || channel_id.is_empty() {
                    return Err("Invalid channel payment".into());
                }

                Ok(true)
            }

            PaymentMethod::Prepaid { api_key, .. } => {
                // TODO: Look up account by API key
                // TODO: Check account has sufficient balance
                // TODO: Deduct from prepaid balance

                if api_key.is_empty() {
                    return Err("Empty API key".into());
                }

                // For now, accept any non-empty API key
                Ok(true)
            }
        }
    }

    /// Get the amount claimed in the payment.
    pub fn claimed_amount(&self) -> u64 {
        match &self.method {
            PaymentMethod::Direct { amount, .. } => *amount,
            PaymentMethod::Channel { amount, .. } => *amount,
            PaymentMethod::Prepaid { .. } => u64::MAX, // Prepaid has unlimited per-request
        }
    }
}

/// Payment verifier with real cryptographic checks.
///
/// Holds runtime state for replay protection, channel tracking, and
/// prepaid account balances. All verification is deterministic and
/// based on actual cryptographic operations.
pub struct PaymentVerifier {
    /// Known public keys for direct payment verification (peer_id -> verifying_key)
    known_keys: RwLock<HashMap<String, VerifyingKey>>,
    /// Replay protection: set of seen proof IDs
    seen_proofs: RwLock<HashSet<String>>,
    /// Payment channel state: channel_id -> (last_nonce, balance, peer_id)
    channels: RwLock<HashMap<String, (u64, u64, String)>>,
    /// Prepaid accounts: api_key -> (account_id, balance)
    prepaid_accounts: RwLock<HashMap<String, (String, u64)>>,
}

impl PaymentVerifier {
    /// Create a new payment verifier with empty state.
    pub fn new() -> Self {
        Self {
            known_keys: RwLock::new(HashMap::new()),
            seen_proofs: RwLock::new(HashSet::new()),
            channels: RwLock::new(HashMap::new()),
            prepaid_accounts: RwLock::new(HashMap::new()),
        }
    }

    /// Register a known public key for direct payment verification.
    pub async fn register_key(&self, peer_id: String, key: VerifyingKey) {
        self.known_keys.write().await.insert(peer_id, key);
    }

    /// Register a payment channel with initial balance.
    pub async fn register_channel(&self, channel_id: &str, balance: u64, peer_id: &str) {
        self.channels
            .write()
            .await
            .insert(channel_id.to_string(), (0, balance, peer_id.to_string()));
    }

    /// Register a prepaid account.
    pub async fn register_prepaid_account(&self, api_key: &str, account_id: &str, balance: u64) {
        self.prepaid_accounts
            .write()
            .await
            .insert(api_key.to_string(), (account_id.to_string(), balance));
    }

    /// Verify a direct payment proof with real ed25519 signature check.
    pub async fn verify_direct(
        &self,
        proof_id: &str,
        amount: u64,
        signature_hex: &str,
        required_amount: u64,
    ) -> Result<bool, String> {
        // Replay protection
        {
            let mut seen = self.seen_proofs.write().await;
            if seen.contains(proof_id) {
                return Err("Proof ID already used (replay detected)".into());
            }
        }

        // Amount check
        if amount < required_amount {
            return Ok(false);
        }

        // Decode signature
        let sig_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
            .map_err(|_| "Invalid signature format")?;
        if sig_bytes.len() != 64 {
            return Err("Signature must be 64 bytes".into());
        }
        let sig = Signature::from_slice(&sig_bytes).map_err(|e| e.to_string())?;

        // The message that was signed is `proof_id:amount`
        let message = format!("{}:{}", proof_id, amount);
        let message_bytes = message.as_bytes();

        // Try to verify against known keys
        let keys = self.known_keys.read().await;
        let mut verified_peer: Option<String> = None;
        for (peer_id, verifying_key) in keys.iter() {
            if verifying_key.verify(message_bytes, &sig).is_ok() {
                verified_peer = Some(peer_id.clone());
                break;
            }
        }
        drop(keys);

        if let Some(peer_id) = verified_peer {
            self.seen_proofs.write().await.insert(proof_id.to_string());
            tracing::debug!(peer_id = %peer_id, proof_id = %proof_id, "Direct payment signature verified");
            return Ok(true);
        }

        // No matching key found — reject
        Err("No matching public key for signature".into())
    }

    /// Verify a channel payment with nonce and balance checks.
    pub async fn verify_channel(
        &self,
        channel_id: &str,
        nonce: u64,
        amount: u64,
        signature_hex: &str,
        required_amount: u64,
    ) -> Result<bool, String> {
        // Amount check
        if amount < required_amount {
            return Ok(false);
        }

        // Decode signature
        let sig_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
            .map_err(|_| "Invalid signature format")?;
        if sig_bytes.len() != 64 {
            return Err("Signature must be 64 bytes".into());
        }
        let sig = Signature::from_slice(&sig_bytes).map_err(|e| e.to_string())?;

        // The message is `(channel_id, nonce, amount)`
        let message = format!("{}:{}:{}", channel_id, nonce, amount);
        let message_bytes = message.as_bytes();

        // Check channel exists and verify nonce + balance
        let mut channels = self.channels.write().await;
        let channel = channels.get_mut(channel_id).ok_or("Channel not found")?;

        let (last_nonce, balance, peer_id) = channel;

        // Nonce must be strictly greater than last seen
        if nonce <= *last_nonce {
            return Err(format!(
                "Nonce {} not greater than last nonce {}",
                nonce, last_nonce
            ));
        }

        // Balance must cover the amount
        if *balance < amount {
            return Err(format!(
                "Insufficient channel balance: have {}, need {}",
                balance, amount
            ));
        }

        // Verify signature against channel peer's known keys
        let keys = self.known_keys.read().await;
        if let Some(verifying_key) = keys.get(peer_id) {
            if verifying_key.verify(message_bytes, &sig).is_err() {
                return Err("Channel signature verification failed".into());
            }
        } else {
            return Err(format!(
                "No public key registered for channel peer {}",
                peer_id
            ));
        }

        // All checks passed — update nonce and deduct balance
        *last_nonce = nonce;
        *balance -= amount;

        tracing::debug!(channel_id = %channel_id, nonce = nonce, "Channel payment verified");
        Ok(true)
    }

    /// Verify a prepaid payment by looking up account and deducting balance.
    pub async fn verify_prepaid(
        &self,
        api_key: &str,
        required_amount: u64,
    ) -> Result<bool, String> {
        if api_key.is_empty() {
            return Err("Empty API key".into());
        }

        let mut accounts = self.prepaid_accounts.write().await;
        let account = accounts.get_mut(api_key).ok_or("Unknown API key")?;

        let (account_id, balance) = account;

        if *balance < required_amount {
            return Err(format!(
                "Insufficient prepaid balance: have {}, need {}",
                from_micro(*balance),
                from_micro(required_amount)
            ));
        }

        *balance -= required_amount;

        tracing::debug!(account_id = %account_id, "Prepaid balance deducted");
        Ok(true)
    }
}

impl Default for PaymentVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert micro-BKG to BKG for display.
fn from_micro(micro: u64) -> f64 {
    micro as f64 / 1_000_000.0
}

/// Response with payment information for 402 responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PaymentInfo {
    /// Required amount in μBKG
    pub required_amount: u64,
    /// Required amount in BKG (for display)
    pub required_amount_bkg: f64,
    /// Wallet address to send payment
    pub payment_address: String,
    /// Supported payment methods
    pub supported_methods: Vec<String>,
    /// Instructions for payment
    pub instructions: String,
}

#[allow(dead_code)]
impl PaymentInfo {
    /// Create payment info for a 402 response.
    pub fn new(required_amount: u64, payment_address: String) -> Self {
        Self {
            required_amount,
            required_amount_bkg: crate::wallet::from_micro(required_amount),
            payment_address,
            supported_methods: vec![
                "direct".to_string(),
                "channel".to_string(),
                "prepaid".to_string(),
            ],
            instructions: "Include payment proof in request headers:\n\
                 - X-Payment-Proof: direct:<proof_id>:<amount>:<signature>\n\
                 - X-Channel-Payment: <channel_id>:<nonce>:<amount>:<signature>\n\
                 - Authorization: Bearer <api_key>"
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_parse_direct_payment() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Payment-Proof",
            HeaderValue::from_static("direct:proof123:1000000:sig456"),
        );

        let proof = PaymentProof::from_headers(&headers).unwrap();

        match proof.method {
            PaymentMethod::Direct {
                proof_id,
                amount,
                signature,
            } => {
                assert_eq!(proof_id, "proof123");
                assert_eq!(amount, 1000000);
                assert_eq!(signature, "sig456");
            }
            _ => panic!("Expected Direct payment method"),
        }
    }

    #[test]
    fn test_parse_channel_payment() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Channel-Payment",
            HeaderValue::from_static("chan_123:5:500000:sig789"),
        );

        let proof = PaymentProof::from_headers(&headers).unwrap();

        match proof.method {
            PaymentMethod::Channel {
                channel_id,
                nonce,
                amount,
                signature,
            } => {
                assert_eq!(channel_id, "chan_123");
                assert_eq!(nonce, 5);
                assert_eq!(amount, 500000);
                assert_eq!(signature, "sig789");
            }
            _ => panic!("Expected Channel payment method"),
        }
    }

    #[test]
    fn test_parse_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer sk_test_12345"),
        );

        let proof = PaymentProof::from_headers(&headers).unwrap();

        match proof.method {
            PaymentMethod::Prepaid { api_key, .. } => {
                assert_eq!(api_key, "sk_test_12345");
            }
            _ => panic!("Expected Prepaid payment method"),
        }
    }

    #[test]
    fn test_verify_sufficient_payment() {
        let proof = PaymentProof {
            method: PaymentMethod::Direct {
                proof_id: "test".to_string(),
                amount: 1000000,
                signature: "valid".to_string(),
            },
            timestamp: 0,
        };

        // Amount is sufficient
        assert!(proof.verify(500000).unwrap());

        // Amount is exactly right
        assert!(proof.verify(1000000).unwrap());

        // Amount is insufficient
        assert!(!proof.verify(2000000).unwrap());
    }
}
