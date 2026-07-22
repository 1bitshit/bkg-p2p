//! BKG P2P Documentation Agent Capability
//!
//! Native documentation lookup, indexing, and search capabilities.
//! Integrates with existing A2A, vector store, safety, and web layers.

pub mod agent;
pub mod provider;
pub mod resolver;
pub mod cache;
pub mod index;
pub mod types;
pub mod audit;

pub use agent::DocumentationAgent;
pub use provider::{DocumentationProvider, LocalDocsProvider, WorkspaceDocsProvider, RustdocProvider, CratesIoDocsProvider, Context7CompatibleProvider};
pub use resolver::LibraryResolver;
pub use cache::DocumentationCache;
pub use index::DocumentationIndex;
pub use types::*;
pub use audit::DocumentationAudit;
