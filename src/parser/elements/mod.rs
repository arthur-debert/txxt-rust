//! Element Parsers
//!
//! This module contains the individual parsers for each txxt element type.
//! Each element parser converts tokens into the appropriate AST node type.
//!
//! Note: Phase 2.b AST Construction modules (annotation, definition, list,
//! paragraph, session, verbatim) have been removed. Only Phase 2.c inline
//! parsing modules remain.

pub mod formatting;
pub mod inlines;
