//! Test Suite Integration
//!
//! This file includes all test modules from the reorganized test structure.
//! This ensures that cargo discovers and runs all tests in subdirectories.
//!
//! ## Test Organization
//!
//! Tests are organized to mirror the source code structure as closely as possible.
//! This ensures that tests are easy to find and maintain, with clear mapping between
//! code and its corresponding tests.
//!
//! ## Adding New Tests
//!
//! When adding new tests:
//!
//! 1. **Identify the Target**: Determine which source module your test is testing
//! 2. **Find the Test Directory**: Look for the corresponding directory in `tests/`
//! 3. **Add Your Test**: Place your test file in the appropriate subdirectory
//! 4. **Update mod.rs**: Add your test module to the relevant `mod.rs` file
//! 5. **Verify Discovery**: Run `cargo test --lib -- --list` to ensure your test is discovered
//!
//! ## Example: Adding a Parser Test
//!
//! If you're testing a new parser element in `src/parser/elements/new_element/`:
//!
//! 1. Create `tests/parser/elements/new_element/new_element_tests.rs`
//! 2. Add `mod new_element_tests;` to `tests/parser/elements/mod.rs`
//! 3. Your test will be automatically discovered by this file
//!
//! ## Test Directory Structure
//!
//! ```
//! tests/
//! ├── suite.rs                    # This file - includes all test modules
//! ├── testing_framework_check.rs  # Only remaining top-level test file
//! │
//! ├── assembler/                  # Tests for assembler components
//! ├── assertions/                # All assertion framework tests
//! ├── ast_elements/               # Tests for AST elements
//! ├── ast_query/                  # Tests for AST query functionality
//! ├── lexer/                      # Tests for lexer components
//! ├── parser/                     # All parser-related tests
//! ├── tools_detokenizer/          # Tests for detokenizer tool
//! ├── tools_treeviz/              # Tests for tree visualization tool
//! ├── verbatim_scanner/           # Tests for verbatim scanner
//! ├── integration/                # Integration and example tests
//! └── infrastructure/             # Shared test infrastructure
//! ```
//!
//! See `docs/dev/file-layout.txxt` for complete documentation.

#![allow(clippy::duplicate_mod)]

// Test directories - organized to mirror the src/ structure
mod assembler;
mod assertions;
mod ast_query;
mod lexer;
mod parser;
// mod tools_detokenizer; // DISABLED: Needs rewrite for Vec<ScannerToken>
mod tools_treeviz;
mod verbatim_scanner;

// Integration and infrastructure
mod infrastructure;
mod integration;

// Individual test files that aren't included elsewhere
mod testing_framework_check;
