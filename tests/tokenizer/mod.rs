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
mod comprehensive_span_audit;
mod debug_parameters;
mod debug_underscore;
mod debug_unicode_positions;
mod debug_whitespace;
mod parameter_span_bug;
mod sequence_marker_span_bug;
mod unicode_span_tests;
mod verify_unicode_handling;
mod whitespace_loss_bug;
