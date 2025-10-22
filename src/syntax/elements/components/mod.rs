//! Shared component elements
//!
//! This module contains shared component elements that are used across
//! multiple other element types, such as labels, parameters, and markers.
//! This mirrors the AST elements/components structure.

pub mod label;
pub mod sequence;
pub mod txxt_marker;

// Re-export main interfaces
pub use label::{parse_label, validate_label, Label, LabelParseResult};
pub use sequence::{read_sequence_marker, SequenceMarkerLexer};
pub use txxt_marker::{read_annotation_marker, read_definition_marker, TxxtMarkerLexer};
