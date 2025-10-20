//! Tools module for TXXT utilities
//!
//! This module contains various tools and utilities for working with TXXT documents,
//! including visualization, analysis, and debugging tools.

// Tree visualization tool for AST inspection and debugging
pub mod treeviz;

// Detokenizer for round-trip verification
// TODO: Update to work with Vec<ScannerToken> instead of removed ScannerTokenTree
// pub mod detokenizer;
