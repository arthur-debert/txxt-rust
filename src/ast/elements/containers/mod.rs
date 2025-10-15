//! Container Elements
//!
//! Container elements implement the core architectural insight of TXXT:
//! "containers are what get indented, not their parent elements."
//!
//! This module contains the generic content container type.
//! Session and ignore containers are now located in their respective functional modules.

pub mod content;

// Re-export container types
pub use content::ContentContainer;
