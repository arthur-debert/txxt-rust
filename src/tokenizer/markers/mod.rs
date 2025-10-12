//! Marker token detection and parsing
//!
//! This module handles the detection and parsing of various marker tokens
//! defined in the TXXT specification, including sequence markers, txxt markers,
//! and reference markers.

pub mod sequence;
// TODO: Implement these modules in subsequent commits
// pub mod txxt_marker;
// pub mod reference;

// Re-export public interfaces
pub use sequence::{read_sequence_marker, SequenceMarkerLexer};
