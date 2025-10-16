//! Test Suite Integration
//!
//! This file includes all test modules from the reorganized test structure.
//! This ensures that cargo discovers and runs all tests in subdirectories.

#![allow(clippy::duplicate_mod)]

// Test directories - organized to mirror the src/ structure
mod assembler;
mod assertions;
mod ast_elements;
mod ast_query;
mod lexer;
mod parser;
mod tools_detokenizer;
mod tools_treeviz;
mod verbatim_scanner;

// Integration and infrastructure
mod infrastructure;
mod integration;

// Individual test files that aren't included elsewhere
mod testing_framework_check;
