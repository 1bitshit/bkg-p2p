use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{Span, field};

/// Telemetry configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub service_name: String,
    pub service_version: String,
    pub export_endpoint: Option<String>,
    pub sample_rate: f64,
    pub max_events: usize,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "bkg-p2p".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            export_endpoint: None,
            sample_rate: 1.0,
            max_events: 10000,
        }
    }
}

/// Telemetry event
#[derive(Debug, Clone, serde::Serialize)]
pub struct TelemetryEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub severity: Severity,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum Severity {
    Debug,
    Info,
    Warn,
    Error,
}

/// Telemetry collector
pub struct Telemetry {
    config: TelemetryConfig,
    events: Arc<RwLock<Vec<TelemetryEvent>>>,
}

impl Telemetry {
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a telemetry event
    pub async fn record(&self, event: TelemetryEvent) {
        if !self.config.enabled {
            return;
        }

        let mut events = self.events.write().await;
        events.push(event);

        // Trim old events if over limit
        if events.len() > self.config.max_events {
            events.drain(0..events.len() - self.config.max_events);
        }
    }

    /// Get recent events
    pub async fn recent(&self, limit: usize) -> Vec<TelemetryEvent> {
        let events = self.events.read().await;
        let start = events.len().saturating_sub(limit);
        events[start..].to_vec()
    }

    /// Clear all events
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        events.clear();
    }
}
