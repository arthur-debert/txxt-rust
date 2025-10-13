//! Element-Specific Parsers
//!
//! This module contains parsers for each TXXT element type, mirroring the
//! structure defined in `docs/specs/elements/` and `src/tokenizer/`.
//!
//! # Organization
//!
//! Each parser module corresponds to:
//! - A specification in `docs/specs/elements/`
//! - A tokenizer in `src/tokenizer/`
//! - AST nodes in `src/ast/nodes/`
//! - Test cases extractable via `TxxtCorpora`
//!
//! # Design Pattern
//!
//! All element parsers follow a consistent pattern:
//! ```rust,ignore
//! pub struct ElementParser;
//!
//! impl ElementParser {
//!     pub fn parse(tokens: &[Token]) -> Result<ElementAst, ParseError> {
//!         // Element-specific parsing logic
//!     }
//! }
//! ```
//!
//! # Testing Integration
//!
//! Each element parser is tested using specification-driven test cases:
//! ```rust,ignore
//! use tests::corpora::{TxxtCorpora, ProcessingStage};
//!
//! let corpus = TxxtCorpora::load_with_processing(
//!     "txxt.core.spec.paragraph.valid.simple",
//!     ProcessingStage::ParsedAst
//! )?;
//! ```

// Block-level element parsers (mirrors docs/specs/elements/)
pub mod annotation;
pub mod container;
pub mod definition;
pub mod labels;
pub mod list;
pub mod paragraph;
pub mod parameters;
pub mod session;
pub mod verbatim;

// Inline element parsers
pub mod inlines;
