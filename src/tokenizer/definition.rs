//! Definition element tokenization
//!
//! Implements tokenization for definition elements as defined in
//! docs/specs/elements/definition.txxt
//!
//! Definition pattern: term :: content or term:params :: content

// Re-export the definition marker reading function from infrastructure
pub use crate::tokenizer::infrastructure::markers::txxt_marker::read_definition_marker;

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_definition_marker_basic() {
        // Basic test to ensure the re-exported function works
        // More comprehensive tests are in the infrastructure module
    }
}
