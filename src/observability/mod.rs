//! BKG P2P Observability Layer
//!
//! Common telemetry, tracing, and metrics for all subsystems.

pub mod telemetry;
pub mod metrics;
pub mod tracing;

pub use telemetry::Telemetry;
pub use metrics::Metrics;
pub use tracing::BkgTracer;
