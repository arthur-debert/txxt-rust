//! TXXT Tokenizer - Character-precise token generation for new AST
//!
//! This module implements the tokenization phase that produces Token enum variants
//! from src/ast/tokens.rs with precise source positioning for language server support.
//!
//! ## Architecture
//!
//! - [`verbatim_scanner`] - Pre-tokenization verbatim block detection
//! - [`lexer`] - Main tokenizer that produces Token enum with SourceSpan positions
//! - [`markers`] - Marker token detection (sequence, txxt, reference markers)
//! - [`inline`] - Inline element parsing (formatting, parameters)
//! - [`indentation`] - Indentation tracking and indent/dedent tokens
//! - [`patterns`] - Shared regex patterns for validation
//!
//! ## New AST Integration
//!
//! This tokenizer is built specifically for the new AST system defined in src/ast/tokens.rs.
//! It produces Token enum variants with precise SourceSpan positioning for character-level
//! language server operations.

pub mod indentation;
pub mod inline;
pub mod lexer;
pub mod markers;
pub mod patterns;
pub mod verbatim_scanner;

pub use lexer::Lexer;
pub use verbatim_scanner::{VerbatimBlock, VerbatimScanner, VerbatimType};

// Re-export marker and inline parsing functionality
pub use inline::{read_inline_delimiter, InlineDelimiterLexer};
pub use markers::{read_sequence_marker, SequenceMarkerLexer};

// Re-export new AST token types
pub use crate::ast::tokens::{Position, SourceSpan, Token, TokenSequence};

/// Main tokenization entry point
///
/// Processes TXXT text and returns Token enum variants with precise source positions
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(text);
    lexer.tokenize()
}
