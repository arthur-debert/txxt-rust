//! Marker infrastructure modules
//!
//! Provides shared infrastructure for marker detection used by
//! specification-aligned tokenizer modules.

pub mod sequence;
pub mod txxt_marker;

// Re-export key marker infrastructure
pub use sequence::SequenceMarkerLexer;
pub use txxt_marker::{
    detect_colon_pattern, integrate_annotation_parameters, integrate_definition_parameters,
    is_start_of_annotation_pattern, ColonPattern, TxxtMarkerLexer,
};
