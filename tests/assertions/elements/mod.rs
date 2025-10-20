//! Element-specific assertion modules
//!
//! Contains assertion functionality organized by element type,
//! following the same structure as the AST and tokenizer modules.

pub mod annotation;
pub mod components;
pub mod definition;
pub mod list;
pub mod paragraph;
pub mod session;
pub mod verbatim;

// Re-export all expected types for convenience
