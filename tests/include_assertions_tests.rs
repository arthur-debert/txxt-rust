//! Assertion framework integration tests
//!
//! Includes all assertion tests from the tests/assertions/ directory structure.
//! This ensures that nested assertion tests are discoverable by cargo test.

#[path = "assertions/mod.rs"]
mod assertions;
