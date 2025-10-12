//! Marker token detection and parsing
//!
//! This module handles the detection and parsing of various marker tokens
//! defined in the TXXT specification, including sequence markers, txxt markers,
//! and reference markers.

pub mod sequence;
pub mod txxt_marker;

// Re-export public interfaces
pub use sequence::{read_sequence_marker, SequenceMarkerLexer};
pub use txxt_marker::{
    detect_colon_pattern, integrate_annotation_parameters, integrate_definition_parameters,
    is_start_of_annotation_pattern, read_annotation_marker, read_definition_marker, ColonPattern,
    TxxtMarkerLexer,
};
