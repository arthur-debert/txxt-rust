#![allow(deprecated)]
//! OBSOLETE: Tests for old VerbatimBlock semantic token transformation
//!
//! These tests use the OLD token names (VerbatimTitle, IndentationWall, IgnoreTextSpan, VerbatimLabel)
//! which were replaced in the verbatim refactor (#132) with new tokens:
//! VerbatimBlockStart, VerbatimContentLine, VerbatimBlockEnd
//!
//! See verbatim_block_v2_transformation.rs for tests using the new tokens.
//!
//! This file has been emptied as the old tests no longer compile with the new token types.
//! The file is kept for git history tracking.

#![cfg(test)]

// All tests have been removed as they use obsolete token types.
// See verbatim_block_v2_transformation.rs for current verbatim transformation tests.
