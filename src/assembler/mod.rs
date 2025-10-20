//! Phase 3: Assembler - Document Assembly
//!
//! This module implements the assembler phase that converts AST element nodes
//! into final document structures.
//!
//! See src/lib.rs for the full architecture overview.
//!
//! - [`document_assembly`] - Step 3.a: Document structure creation
//!   - Wraps AST elements in Session container and Document node
//!   - Input: Vec<ElementNode>
//!   - Output: Document with metadata and assembly info
//!   - Creates proper document hierarchy with Session containers
//!
//! - [`annotation_attachment`] - Step 3.b: Annotation proximity-based attachment
//!   - Attaches annotations to their target elements using proximity rules
//!   - Input: Document with unattached annotations
//!   - Output: Document with annotations properly attached
//!   - Handles document-level and content-level annotation processing

// Processing steps
pub mod annotation_attachment;
pub mod document_assembly;

// Re-export main interfaces
pub use annotation_attachment::{AnnotationAttacher, AnnotationAttachmentError};
pub use document_assembly::{DocumentAssembler, DocumentAssemblyError};
