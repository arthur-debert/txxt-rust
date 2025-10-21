//! Annotation element tokenization
//!
//! Implements tokenization for annotation elements as defined in
//! docs/specs/elements/annotation/annotation.txxt
//!
//! Annotation pattern: :: label :: content or :: label:params :: content

// Re-export the annotation marker reading function from infrastructure
pub use crate::syntax::elements::components::txxt_marker::read_annotation_marker;
