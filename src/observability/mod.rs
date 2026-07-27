//! BKG P2P Observability Layer
//!
//! Common telemetry, tracing, and metrics for all subsystems.

pub mod metrics;
pub mod telemetry;
pub mod tracing;

pub use metrics::Metrics;
pub use telemetry::Telemetry;
pub use tracing::BkgTracer;
