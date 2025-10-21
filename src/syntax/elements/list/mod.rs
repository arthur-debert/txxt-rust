//! List element tokenization
//!
//! Implements tokenization for list elements as defined in
//! docs/specs/elements/list/list.txxt
//!
//! Sequence marker parsing for lists

pub mod list_tokenizer;

// Re-export main interfaces
pub use crate::syntax::elements::components::sequence::read_sequence_marker;
