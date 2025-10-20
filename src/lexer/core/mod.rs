//! Core tokenization support
//!
//! This module contains supporting components used by the tokenization step.
//!
//! ## Core Components
//!
//! - [`indentation`] - Indentation tracking and container boundary detection
//! - [`patterns`] - Core pattern matching utilities for token recognition

pub mod indentation;
pub mod patterns;

// Re-export main interfaces
pub use indentation::{IndentationTracker, INDENT_SIZE, TAB_WIDTH};
pub use patterns::*;
