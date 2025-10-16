//! Core tokenization logic
//!
//! This module contains the fundamental tokenization components that are
//! not element-specific but provide the core functionality for converting
//! TXXT source text into tokens. This mirrors the AST/parser core structure.
//!
//! ## Core Components
//!
//! - [`lexer`] - Main tokenization engine that processes source text
//! - [`indentation`] - Indentation tracking and container boundary detection
//! - [`patterns`] - Core pattern matching utilities for token recognition

pub mod indentation;
pub mod lexer;
pub mod patterns;

// Re-export main interfaces
pub use indentation::{IndentationTracker, INDENT_SIZE, TAB_WIDTH};
pub use lexer::Lexer;
pub use patterns::*;
