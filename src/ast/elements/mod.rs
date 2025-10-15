//! AST Elements - Spec-Aligned Structure
//!
//! This module defines AST elements with perfect alignment to the specification
//! structure in `docs/specs/elements/`. The file organization mirrors:
//!
//! - `docs/specs/elements/` - Specification documents
//! - `src/parser/elements/` - Parser modules  
//! - `src/ast/elements/` - AST node definitions (this module)
//! - `tests/ast/elements/` - AST-specific test modules
//!
//! # Taxonomy Alignment
//!
//! The structure follows the terminology hierarchy from `docs/specs/core/terminology.txxt`:
//!
//! ## Span Elements (Inline)
//! - Text spans, formatting spans, reference spans
//! - Cannot contain line breaks
//! - Located in `inlines/` submodules
//!
//! ## Line Elements  
//! - Text lines, blank lines
//! - Encompass full lines of text
//! - May contain multiple spans
//!
//! ## Block Elements
//! - Paragraphs, lists, definitions, verbatim, sessions, annotations
//! - Contain one or more lines
//! - Primary structural units
//!
//! ## Container Elements
//! - Content containers, session containers, ignore containers
//! - Hold child elements of different types
//! - What gets indented in the format

// Core element trait and types
pub mod core;

// Core AST files
pub mod blocks;
pub mod tokens;
pub mod traversal;

// Container elements (hold child elements)
pub mod containers;

// Block-level elements
pub mod annotation;
pub mod definition;
pub mod list;
pub mod paragraph;
pub mod session;
pub mod verbatim;

// Document-level elements
pub mod document;

// Formatting elements
pub mod formatting;

// Reference elements
pub mod references;

// Inline/span elements
pub mod inlines;

// Shared component elements
pub mod components;

// Re-export core types for convenience
pub use core::{ElementType, TxxtElement};
