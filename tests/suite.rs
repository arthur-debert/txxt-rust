//! Test Suite Integration
//!
//! This file includes all test modules from the reorganized test structure.
//! This ensures that cargo discovers and runs all tests in subdirectories.

#![allow(clippy::duplicate_mod)]

// Individual test files that aren't included elsewhere
mod assert_paragraph_complete_tests;
mod assertion_framework_tests;
mod component_assertion_tests;
mod detokenizer_tests;
mod ensemble_documents_example;
mod illustrated_parser_test_example;
mod include_assembler_tests;
mod include_assertions_tests;
mod include_ast_tests;
mod include_lexer_tests;
mod issue_26_indented_sequence_markers;
mod parser_integration;
mod parser_tests;
mod testing_framework_check;
mod tools_detokenizer;
mod tools_treeviz;
