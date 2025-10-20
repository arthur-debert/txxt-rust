//! Lexer test suite
//!
//! Organized to mirror the src/lexer/ module structure for clear mapping
//! between code and tests. Follows the specification-aligned organization.

// Specification-aligned test modules
mod annotation;
mod definition;
mod indentation;
mod list;
mod parameters;

// Infrastructure and core tests
mod core;
mod debug;
mod escape_sequences;
mod inline;
mod pipeline;
mod verbatim;

// Bug reproduction tests
mod blankline_whitespace;
mod comprehensive_span_audit;
mod debug_parameters;
mod debug_underscore;
mod debug_unicode_positions;
mod debug_whitespace;
mod parameter_span_bug;
mod sequence_marker_span_bug;
mod unicode_regression_tests;
mod unicode_span_tests;
mod verbatim_false_positive;
mod verify_unicode_handling;
mod whitespace_loss_bug;
