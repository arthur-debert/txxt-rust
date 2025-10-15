//! Document Elements
//!
//! Document-level elements for top-level document structure.

pub mod document_structure;

// Re-export document types
pub use document_structure::{AssemblyInfo, Document, Meta, MetaValue, ProcessingStats};
