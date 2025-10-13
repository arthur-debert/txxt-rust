//! Tokenizer test suite
//!
//! Organized to mirror the src/tokenizer/ module structure for clear mapping
//! between code and tests. Follows the specification-aligned organization.

// Specification-aligned test modules
mod annotation;
mod container;
mod definition;
mod indentation;
mod labels;
mod list;
mod paragraph;
mod parameters;
mod session;

// Infrastructure and core tests
mod core;
mod inline;
mod verbatim;

// Bug reproduction tests
mod debug_whitespace;
mod whitespace_loss_bug;
