//! Container Elements
//!
//! Container elements implement the core architectural insight of TXXT:
//! "containers are what get indented, not their parent elements."
//!
//! This module contains the three container types defined in the specification:
//! - Content containers: Cannot contain sessions
//! - Session containers: Can contain sessions  
//! - Ignore containers: Verbatim content only

pub mod content;
pub mod ignore;
pub mod session;

// Re-export container types
pub use content::ContentContainer;
pub use ignore::IgnoreContainer;
pub use session::SessionContainer;
