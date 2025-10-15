//! Annotation Elements
//!
//! Annotation elements for marking up document content with metadata,
//! comments, and other non-content information.

pub mod block;

// Re-export annotation types
pub use block::{AnnotationBlock, AnnotationContent};
