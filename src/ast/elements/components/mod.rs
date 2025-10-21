//! Component Elements
//!
//! Shared component elements used across different element types.

pub mod label;
pub mod labels;
pub mod parameters;

// Re-export component types
pub use label::ParsedLabel;
pub use labels::Label;
pub use parameters::Parameters;
