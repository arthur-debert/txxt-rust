//! TXXT Tokenizer - Perfect specification alignment
//!
//! This module implements tokenization with perfect 1:1 alignment to the TXXT specification.
//! Every tokenizer module corresponds directly to a specification file in docs/specs/elements/.
//!
//! ## Perfect Specification Alignment:
//!
//!   docs/specs/elements/                    src/tokenizer/
//!   ├── annotation.txxt                →    ├── annotation.rs
//!   ├── container.txxt                 →    ├── container.rs  
//!   ├── definition.txxt                →    ├── definition.rs
//!   ├── list.txxt                      →    ├── list.rs
//!   ├── paragraph.txxt                 →    ├── paragraph.rs
//!   ├── session.txxt                   →    ├── session.rs
//!   ├── verbatim.txxt                  →    ├── verbatim_scanner.rs
//!   ├── labels.txxt                    →    ├── labels.rs
//!   ├── parameters.txxt                →    ├── parameters.rs
//!   └── inlines/                            └── inline/
//!       ├── inlines-general.txxt       →        ├── general.rs
//!       ├── formatting.txxt            →        ├── formatting/
//!       │                                       │   ├── delimiters.rs
//!       │                                       │   └── math_span.rs
//!       └── references/                          └── references/
//!           ├── references-general.txxt →            ├── general.rs
//!           └── citations.txxt          →            ├── citations.rs
//!                                                    ├── page_ref.rs
//!                                                    └── session_ref.rs
//!
//!   Infrastructure (non-specification modules):
//!   └── infrastructure/
//!       ├── lexer.rs                  # Main tokenizer engine
//!       ├── patterns.rs               # Pattern utilities
//!       └── markers/                  # Shared marker infrastructure
//!           ├── sequence.rs           # Used by list.rs
//!           └── txxt_marker.rs        # Used by annotation.rs, definition.rs
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

// Specification-aligned modules (1:1 with docs/specs/elements/)
pub mod annotation;
pub mod container;
pub mod definition;
pub mod indentation;
pub mod inline;
pub mod labels;
pub mod list;
pub mod paragraph;
pub mod parameters;
pub mod session;
pub mod verbatim_scanner;

// Infrastructure modules (non-specification)
pub mod infrastructure;

// Re-export key components
pub use indentation::IndentationTracker;
pub use infrastructure::lexer::Lexer;
pub use verbatim_scanner::{VerbatimBlock, VerbatimScanner, VerbatimType};

// Re-export new AST token types
pub use crate::ast::tokens::{Position, SourceSpan, Token, TokenSequence};

/// Main tokenization entry point
///
/// Processes TXXT text and returns Token enum variants with precise source positions
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut lexer = infrastructure::lexer::Lexer::new(text);
    lexer.tokenize()
}
