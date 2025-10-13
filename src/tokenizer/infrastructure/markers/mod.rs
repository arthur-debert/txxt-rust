//! Marker detection infrastructure
//!
//! Contains low-level marker detection functions that are used by
//! specification-aligned modules.

pub mod parameter_integration;
pub mod parameter_integration_v2;
pub mod sequence;
pub mod txxt_marker;

// Re-export main interfaces
pub use parameter_integration::{
    integrate_annotation_parameters_fixed, integrate_definition_parameters_fixed,
};
pub use parameter_integration_v2::{
    integrate_annotation_parameters_v2, integrate_definition_parameters_v2,
};
pub use sequence::{read_sequence_marker, SequenceMarkerLexer};
pub use txxt_marker::{read_annotation_marker, read_definition_marker, TxxtMarkerLexer};
