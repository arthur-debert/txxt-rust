//! Phase 1: Lexer - Tokenization
//!
//! This module implements the lexer phase that converts source text into scanner tokens.
//!
//! See src/lib.rs for the full architecture overview.
//!
//! ## Lexer Steps
//!
//! - [`verbatim_scanning`] - Step 1.a: Verbatim region identification
//!   - Identifies and marks verbatim regions before tokenization
//!   - Input: Raw source text
//!   - Output: Text with verbatim boundaries marked
//!
//! - [`tokenization`] - Step 1.b: Character-precise tokenization
//!   - Converts text to flat scanner token stream with source positions
//!   - Input: Source text with verbatim markers
//!   - Output: Vec<ScannerToken>
//!
//! - [`semantic_analysis`] - Step 1.c: High-level token analysis
//!   - Converts scanner tokens to high-level tokens
//!   - Input: Vec<ScannerToken>
//!   - Output: HighLevelTokenList
//!
//! ## Supporting Modules
//!
//! - [`core`] - Fundamental tokenization components (indentation tracking, patterns)
//! - [`elements`] - Element-specific tokenization logic organized by specification

// Processing steps
pub mod semantic_analysis;
pub mod tokenization;
pub mod verbatim_scanning;

// Supporting modules
pub mod block_grouping;
pub mod core;
pub mod elements;
pub mod indentation_analysis;
pub mod line_classification;
pub mod parameter_parsing;
pub mod verbatim_boundary;

// Re-export main interfaces
pub use semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};
pub use tokenization::Lexer;
pub use verbatim_scanning::{VerbatimBlock, VerbatimScanner, VerbatimType};

// Re-export line classification functions
pub use line_classification::{is_blank_line, is_definition_marker};

// Re-export formatting functionality
pub use elements::formatting::{read_inline_delimiter, InlineDelimiterLexer};

// Re-export reference functionality
pub use elements::references::{
    read_citation_ref, read_page_ref, read_session_ref, CitationRefLexer, PageRefLexer,
    ReferenceLexer, SessionRefLexer,
};

// Re-export new AST scanner token types
pub use crate::cst::{Position, ScannerToken, ScannerTokenSequence, SourceSpan};

/// Main tokenization entry point
///
/// Processes TXXT text and returns ScannerToken enum variants with precise source positions
pub fn tokenize(text: &str) -> Vec<ScannerToken> {
    let mut lexer = Lexer::new(text);
    lexer.tokenize()
}
