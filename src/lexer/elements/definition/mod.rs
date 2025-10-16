//! Definition element tokenization
//!
//! Implements tokenization for definition elements as defined in
//! docs/specs/elements/definition/definition.txxt
//!
//! Definition pattern: term :: content or term:params :: content

pub mod definition_tokenizer;

// Re-export main interfaces
pub use crate::lexer::elements::components::txxt_marker::read_definition_marker;
