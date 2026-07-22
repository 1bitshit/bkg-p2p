use tracing::{Span, field};

/// BKG P2P Tracer wrapper
pub struct BkgTracer {
    service_name: String,
}

impl BkgTracer {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// Create a new span
    pub fn span(&self, name: &str) -> Span {
        tracing::info_span!(
            name,
            service = %self.service_name,
        )
    }

    /// Create a span with attributes
    pub fn span_with_attrs(&self, name: &str, attrs: Vec<(&str, String)>) -> Span {
        let span = tracing::info_span!(
            name,
            service = %self.service_name,
        );

        for (key, value) in attrs {
            span.record(key, value);
        }

        span
    }
}
