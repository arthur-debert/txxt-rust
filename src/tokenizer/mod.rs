//! TXXT Tokenizer - Character-precise token generation for new AST
//!
//! This module implements the tokenization phase that produces Token enum variants
//! from src/ast/tokens.rs with precise source positioning for language server support.
//!
//! ## File Layout/ In progress, do not break:
//!
//!   src/tokenizer/
//!   ├── mod.rs                    # Public API
//!   ├── lexer.rs                  # Main tokenizer
//!   ├── indentation.rs            # Indent/dedent tracking
//!   ├── verbatim_scanner.rs       # Pre-parsing verbatim detection (existing)
//!   ├── markers/
//!   │   ├── mod.rs               # Marker token detection
//!   │   ├── sequence.rs          # List sequence markers
//!   │   ├── txxt_marker.rs       # :: token detection and classification
//!   │   └── reference.rs         # Reference bracket parsing
//!   ├── inline/
//!   │   ├── mod.rs               # Inline element detection
//!   │   ├── formatting.rs        # Bold, italic, code, math delimiters
//!   │   ├── math_span.rs         # Math expression spans (#content#)
//!   │   ├── citation_ref.rs      # Citation references ([@key])
//!   │   ├── page_ref.rs          # Page references ([p.123])
//!   │   ├── session_ref.rs       # Session references ([#1.2])
//!   │   └── parameters.rs        # Parameter parsing (key=value lists)
//!   └── patterns.rs             # Pattern matching and content extraction utilities
//!
//! ## Architecture
//!
//! - [`verbatim_scanner`] - Pre-tokenization verbatim block detection
//! - [`lexer`] - Main tokenizer that produces Token enum with SourceSpan positions
//! - [`markers`] - Marker token detection (sequence, txxt, reference markers)
//! - [`inline`] - Inline element parsing (formatting)
//! - [`parameters`] - Parameter parsing (key=value syntax)
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
pub use inline::{parse_parameters, read_inline_delimiter, InlineDelimiterLexer, ParameterLexer};
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
