//! Session Elements
//!
//! Session-related elements including sessions, session containers, and headings.

pub mod block;
pub mod session_container;

// Re-export session types
pub use block::{SessionBlock, SessionNumbering, SessionTitle};
pub use session_container::SessionContainer;
