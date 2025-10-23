//! Element Parsers
//!
//! This module contains the individual parsers for each txxt element type.
//! Each element parser converts tokens into the appropriate AST node type.
//!
//! ## Organization
//!
//! Block-level element construction (Phase 2.a):
//! - `annotation` - Annotation block construction
//! - `definition` - Definition block construction
//! - `list` - List block construction
//! - `paragraph` - Paragraph block construction
//! - `session` - Session block construction
//! - `verbatim` - Verbatim block construction
//!
//! Inline-level element parsing (Phase 2.b):
//! - `formatting` - Text formatting elements (bold, italic, code, math)
//! - `inlines` - Inline elements and references
//!
//! Reusable component constructors:
//! - `parameters` - Single source of truth for parameter AST construction

// Block-level element construction
pub mod annotation;
pub mod definition;
pub mod list;
pub mod paragraph;
pub mod session;
pub mod verbatim;

// Inline-level element parsing
pub mod formatting;
pub mod inlines;

// Reusable component constructors
pub mod numbering;
pub mod parameters;
