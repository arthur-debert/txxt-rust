//! TXXT Tokenizer - Perfect specification alignment
//!
//! Every tokenizer module corresponds directly to a specification file in
//! docs/specs/elements/, enabling intuitive navigation from specification to
//! implementation. Infrastructure modules are separated to infrastructure/
//! for clear architectural boundaries.
//!
//! ## Specification-Aligned Modules
//!
//! - [`annotation`] - docs/specs/elements/annotation.txxt
//! - [`definition`] - docs/specs/elements/definition.txxt
//! - [`list`] - docs/specs/elements/list.txxt
//! - [`parameters`] - docs/specs/elements/parameters.txxt
//! - [`container`] - docs/specs/elements/container.txxt (TODO)
//! - [`labels`] - docs/specs/elements/labels.txxt (TODO)
//! - [`paragraph`] - docs/specs/elements/paragraph.txxt (TODO)
//! - [`session`] - docs/specs/elements/session.txxt (TODO)
//! - [`inline`] - docs/specs/elements/inlines/ (complete)
//!
//! ## Infrastructure Modules
//!
//! - [`infrastructure`] - Lexer, patterns, and marker detection
//! - [`verbatim_scanner`] - Pre-parsing verbatim detection
//! - [`indentation`] - Indent/dedent tracking (TODO implementation)
//!
//! ## Architecture
//!
//! This design achieves perfect 1:1 mapping between specification and implementation
//! while maintaining clear separation between infrastructure and specification-driven code.

// Specification-aligned modules
pub mod annotation;
pub mod container;
pub mod definition;
pub mod indentation;
pub mod labels;
pub mod list;
pub mod paragraph;
pub mod parameters;
pub mod session;

// Infrastructure and utilities
pub mod infrastructure;
pub mod inline;
pub mod verbatim_scanner;

// Re-export main interfaces
pub use infrastructure::lexer::Lexer;
pub use verbatim_scanner::{VerbatimBlock, VerbatimScanner, VerbatimType};

// Re-export specification-aligned functions
pub use annotation::read_annotation_marker;
pub use definition::read_definition_marker;
pub use list::read_sequence_marker;
pub use parameters::{parse_parameters, ParameterLexer};

// Re-export inline functionality
pub use inline::{
    read_citation_ref, read_inline_delimiter, read_page_ref, read_session_ref, CitationRefLexer,
    InlineDelimiterLexer, PageRefLexer, ReferenceLexer, SessionRefLexer,
};

// Re-export new AST token types
pub use crate::ast::tokens::{Position, SourceSpan, Token, TokenSequence};

/// Main tokenization entry point
///
/// Processes TXXT text and returns Token enum variants with precise source positions
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(text);
    lexer.tokenize()
}
