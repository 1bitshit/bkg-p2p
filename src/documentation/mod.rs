//! BKG P2P Documentation Agent Capability
//!
//! Native documentation lookup, indexing, and search capabilities.
//! Integrates with existing A2A, vector store, safety, and web layers.

pub mod agent;
pub mod audit;
pub mod cache;
pub mod index;
pub mod provider;
pub mod resolver;
pub mod types;

pub use agent::DocumentationAgent;
pub use audit::DocumentationAudit;
pub use cache::DocumentationCache;
pub use index::DocumentationIndex;
pub use provider::{
    Context7CompatibleProvider, CratesIoDocsProvider, DocumentationProvider, LocalDocsProvider,
    RustdocProvider, WorkspaceDocsProvider,
};
pub use resolver::LibraryResolver;
pub use types::*;
