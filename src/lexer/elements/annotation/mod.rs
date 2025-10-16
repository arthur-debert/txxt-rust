//! Annotation element tokenization
//!
//! Implements tokenization for annotation elements as defined in
//! docs/specs/elements/annotation/annotation.txxt
//!
//! Annotation pattern: :: label :: content or :: label:params :: content

pub mod annotation_tokenizer;

// Re-export main interfaces
pub use crate::lexer::elements::components::txxt_marker::read_annotation_marker;
