//! Verbatim element tokenization
//!
//! Implements tokenization for verbatim elements as defined in
//! docs/specs/elements/verbatim/verbatim.txxt
//!
//! Verbatim elements preserve exact formatting and content without interpretation.
//! This includes code blocks, raw text, and other literal content.

pub mod verbatim_scanner;

// Re-export main interfaces
pub use verbatim_scanner::{VerbatimBlock, VerbatimScanner, VerbatimType};
