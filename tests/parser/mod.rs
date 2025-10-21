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
