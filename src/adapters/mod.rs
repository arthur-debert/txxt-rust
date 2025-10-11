//! Adapter modules for bridging between different AST systems
//!
//! This module provides adapters to bridge between the legacy string-based AST
//! and the new type-safe AST during the transition period. The adapters enable
//! gradual migration of parser components while maintaining compatibility.

#[cfg(feature = "new-ast")]
pub mod old_to_new_ast;

#[cfg(feature = "new-ast")]
pub use old_to_new_ast::convert_document;
