//! Annotation Elements
//!
//! Annotation elements for marking up document content with metadata,
//! comments, and other non-content information.

pub mod annotation_block;
pub mod annotation_content;

// Re-export annotation types
pub use annotation_block::{AnnotationBlock, AnnotationContent};
pub use annotation_content::*;
