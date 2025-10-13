//! AST Node Definitions
//!
//! This module contains AST node definitions that mirror the element structure
//! defined in `docs/specs/elements/`. Each node type corresponds to a specific
//! TXXT element and provides a typed representation for that element.
//!
//! # Organization
//!
//! The module structure exactly mirrors:
//! - `docs/specs/elements/` - Specification documents
//! - `src/tokenizer/` - Tokenizer modules  
//! - `src/parser/elements/` - Parser modules
//!
//! # Design Principles
//!
//! - **Type Safety**: Each element has a specific AST node type
//! - **Token Precision**: All nodes maintain source position information
//! - **Spec Alignment**: Node structure matches specification exactly
//! - **Parser Integration**: Nodes are designed for easy parser construction

// Block-level element AST nodes
pub mod annotation;
pub mod container;
pub mod definition;
pub mod list;
pub mod paragraph;
pub mod session;
pub mod verbatim;

// Inline element AST nodes
pub mod inlines;
