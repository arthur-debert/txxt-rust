//! Definition element tokenization
//!
//! Implements tokenization for definition elements as defined in
//! docs/specs/elements/definition/definition.txxt
//!
//! Definition pattern: term :: content or term:params :: content

// Re-export the definition marker reading function from infrastructure
pub use crate::lexer::elements::components::txxt_marker::read_definition_marker;
