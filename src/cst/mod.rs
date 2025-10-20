//! Concrete Syntax Tree (CST) - Token-level representation
//!
//! The CST layer represents the token-level structure of TXXT documents,
//! preserving exact source locations and syntactic details. This is distinct
//! from the AST which represents semantic structure.
//!
//! # Architecture
//!
//! The CST is organized into three layers:
//!
//! 1. **Primitives** - Core types used across all tokens
//!    - `Position`: Character-precise (row, column) location
//!    - `SourceSpan`: Range of source positions
//!    - `ScannerTokenSequence`: Ordered collection of tokens
//!
//! 2. **Scanner Tokens** - Low-level character-precise tokens
//!    - `ScannerToken`: Flat token stream from lexer
//!    - Individual token types (text, whitespace, punctuation, etc.)
//!
//! 3. **High-Level Tokens** - Semantic grouping of scanner tokens
//!    - `HighLevelToken`: Line-level syntactic structures
//!    - Bridges scanner tokens and AST elements

pub mod high_level_tokens;
pub mod primitives;
pub mod scanner_tokens;

// Re-export core types for convenience
pub use high_level_tokens::{
    FromScannerToken, HighLevelNumberingForm, HighLevelNumberingStyle, HighLevelToken,
    HighLevelTokenBuilder, HighLevelTokenList, HighLevelTokenSpan, ToScannerToken,
};
pub use primitives::{Position, ScannerTokenSequence, SourceSpan};
pub use scanner_tokens::{ScannerToken, SequenceMarkerType, WallType};
