//! Marker detection infrastructure
//!
//! Contains low-level marker detection functions that are used by
//! specification-aligned modules.

pub mod sequence;
pub mod txxt_marker;

// Re-export main interfaces
pub use sequence::{read_sequence_marker, SequenceMarkerLexer};
pub use txxt_marker::{read_annotation_marker, read_definition_marker, TxxtMarkerLexer};
