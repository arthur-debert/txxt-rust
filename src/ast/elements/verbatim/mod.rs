//! Verbatim Elements
//!
//! Verbatim elements for preserving exact formatting and content.

pub mod block;
pub mod ignore_container;

// Re-export verbatim types
pub use block::{VerbatimBlock, VerbatimType};
pub use ignore_container::IgnoreContainer;
