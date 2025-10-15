//! List element tokenization
//!
//! Implements tokenization for list elements as defined in
//! docs/specs/elements/list.txxt
//!
//! Sequence marker parsing for lists

// Re-export the sequence marker reading function from infrastructure
pub use crate::tokenizer::elements::components::sequence::read_sequence_marker;
