//! Container Elements
//!
//! Container elements implement the core architectural insight of TXXT:
//! "containers are what get indented, not their parent elements."
//!
//! This module contains container types:
//! - ContentContainer: Holds any blocks except sessions (list items, etc.)
//! - SimpleContainer: Only basic blocks - paragraphs, lists, verbatim (definitions, annotations)
//!
//! Session and ignore containers are located in their respective functional modules.

pub mod content;
pub mod simple;

// Re-export container types
pub use content::{ContentContainer, ContentContainerElement};
pub use simple::{SimpleBlockElement, SimpleContainer};
