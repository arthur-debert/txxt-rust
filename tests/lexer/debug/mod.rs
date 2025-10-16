//! Debug tests for development and troubleshooting
//!
//! These tests are used during development to debug specific tokenizer
//! functionality. They provide detailed output and are useful for
//! understanding tokenizer behavior during development.
//!
//! Note: These tests may be verbose and are primarily for debugging,
//! not for CI validation.

// Verbatim-related debug tests
mod debug_false_starts;
mod debug_multiple_blocks;
mod debug_stretched;
mod debug_stretched_both;
mod debug_stretched_indented;
mod debug_verbatim_scanner;
mod debug_verbatim_terminator;
mod debug_verbatim_title_fix;

// Core tokenization debug tests
mod debug_blankline_tokenization;
mod debug_definition_marker;
mod debug_parameters;
mod debug_regex_patterns;
