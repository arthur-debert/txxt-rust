//! Assembler Pipeline Components
//!
//! This module implements Phase 3 of the TXXT parsing pipeline that converts
//! AST element nodes into final document structures.
//!
//! # Phase 3: Assembly (AST Tree -> Document)
//!
//! ## 3.a Document Assembly
//! - Wraps AST tree in Session container and Document node
//! - Creates proper document hierarchy with metadata
//! - **Input**: AST tree of `ElementNode` variants (from Phase 2b)
//! - **Output**: `Document` with proper hierarchy and metadata
//!
//! ## 3.b Annotation Attachment  
//! - Applies proximity rules to attach annotations to their targets
//! - Handles document-level and element-level annotation attachment
//! - **Input**: `Document` with unprocessed annotations (from Phase 3a)
//! - **Output**: `Document` with annotations properly attached
//!
//! # Pipeline Stages
//!
//! - [`document_assembly`] - Phase 3a: Document structure creation
//! - [`annotation_attachment`] - Phase 3b: Annotation processing
//!
//! # Current Status
//!
//! Phase 3 is currently stubbed out with placeholder implementations.
//! The actual assembly logic will be implemented as the AST element types
//! and assembly requirements are finalized.

pub mod annotation_attachment;
pub mod document_assembly;

// Re-export main interfaces
pub use annotation_attachment::{AnnotationAttacher, AnnotationAttachmentError};
pub use document_assembly::{DocumentAssembler, DocumentAssemblyError};
