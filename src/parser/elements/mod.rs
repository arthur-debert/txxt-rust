//! Element Parsers
//!
//! This module contains the individual parsers for each txxt element type.
//! Each element parser converts tokens into the appropriate AST node type.

pub mod annotation;
pub mod definition;
pub mod inlines;
pub mod list;
pub mod paragraph;
pub mod session;
pub mod verbatim;
