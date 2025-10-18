//! Assembler Pipeline Components
//!
//! This module implements Phase 3 of the TXXT parsing pipeline that converts
//! AST element nodes into final document structures.
//!
//! src/parser/mod.rs has the full architecture overview.

pub mod annotation_attachment;
pub mod document_assembly;

// Re-export main interfaces
pub use annotation_attachment::{AnnotationAttacher, AnnotationAttachmentError};
pub use document_assembly::{DocumentAssembler, DocumentAssemblyError};
