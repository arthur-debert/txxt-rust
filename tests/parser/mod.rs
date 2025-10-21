//! Parser tests
mod ast_construction;
mod elements;
mod semantic_analysis;

// Parser integration and bug tests
mod issue_26_indented_sequence_markers;
// TODO: Update for new API
// mod parser_integration;
mod parser_tests;

// Ensemble document tests (regex-based grammar engine)
mod ensemble_01_two_paragraphs;
mod ensemble_02_session_one_paragraph;
mod ensemble_03_session_multiple_paragraphs;
mod ensemble_04_multiple_sessions_flat;
mod ensemble_05_nested_sessions_basic;
mod ensemble_06_nested_sessions_multiple;
mod ensemble_session_with_list;

// List parsing tests (simple to complex)
mod list_01_simple_single;
mod test_list_simple;

// Debug utilities
mod debug_tokens;
