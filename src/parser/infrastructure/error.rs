//! Parser Error Types and Handling
//!
//! Comprehensive error handling for all parser phases with
//! helpful error messages and recovery strategies.

/// Unified parser error type
#[derive(Debug, Clone)]
pub enum ParserError {
    /// Lexer errors (Phase 1)
    Lexer(String),
    /// Block grouping errors (Phase 2a)
    BlockGrouping(String),
    /// Parsing errors (Phase 2b)
    Parsing(String),
    /// Post-processing errors (Phase 3)
    PostProcessing(String),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::Lexer(msg) => write!(f, "Lexer error: {}", msg),
            ParserError::BlockGrouping(msg) => write!(f, "Block grouping error: {}", msg),
            ParserError::Parsing(msg) => write!(f, "Parsing error: {}", msg),
            ParserError::PostProcessing(msg) => write!(f, "Post-processing error: {}", msg),
        }
    }
}

impl std::error::Error for ParserError {}
