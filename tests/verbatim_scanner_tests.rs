//! Verbatim Scanner Integration Tests
//!
//! This module runs comprehensive tests for the verbatim scanner using
//! fixture files with embedded test expectations.

mod verbatim_scanner;

// Re-export the integration test
pub use verbatim_scanner::test_runner::tests::test_verbatim_scanner_integration;
