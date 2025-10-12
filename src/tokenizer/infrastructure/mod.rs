//! Infrastructure modules for tokenization
//!
//! These modules provide the foundational infrastructure for tokenization
//! but do not correspond directly to specification elements.

pub mod lexer;
pub mod markers;
pub mod patterns;

// Re-export key infrastructure components
pub use lexer::{Lexer, LexerState};
pub use patterns::*;
