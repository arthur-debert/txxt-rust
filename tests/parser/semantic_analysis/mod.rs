#![allow(deprecated)]
//! Tests for semantic analysis transformations
//!
//! This module contains tests for the semantic analysis phase that transforms
//! scanner tokens into semantic tokens. Each transformation has its own test file.
//!
//! src/parser/mod.rs has the full architecture overview.

pub mod annotation_transformation;
pub mod definition_transformation;
pub mod integration_bug_tests;
pub mod label_transformation;
pub mod parameter_transformation;
pub mod plain_text_line_transformation;
pub mod sequence_marker_transformation;
pub mod sequence_text_line_transformation;
pub mod text_span_transformation;
pub mod txxt_marker_transformation;
pub mod verbatim_block_transformation;
pub mod verbatim_block_v2_transformation;
