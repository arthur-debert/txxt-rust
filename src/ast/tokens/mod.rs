//! Organized token modules for TXXT parsing pipeline
//!
//! DEPRECATED: This module is kept for backward compatibility.
//! All tokens have been moved to crate::cst (Concrete Syntax Tree).
//!
//! Use `crate::cst` for all new code.

// Re-export from CST for backward compatibility
pub use crate::cst::*;

// Legacy scanner token type modules (kept for compatibility, not actively used)
pub mod content;
pub mod formatting;
pub mod ignore;
pub mod markers;
pub mod punctuation;
pub mod references;
pub mod structural;

// High-level tokens now in CST
pub mod high_level {
    //! DEPRECATED: Use crate::cst::high_level_tokens instead
    pub use crate::cst::high_level_tokens::*;
}

// Re-export individual token types for backward compatibility
pub use content::*;
pub use formatting::*;
pub use ignore::*;
pub use markers::*;
pub use punctuation::*;
pub use references::*;
pub use structural::*;
