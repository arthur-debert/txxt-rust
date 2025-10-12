//! Sequence marker parsing for lists
//!
//! Handles detection and parsing of list sequence markers as defined in the
//! TXXT specification, including:
//! - Plain markers: "- "
//! - Numerical markers: "1. ", "42. "
//! - Alphabetical markers: "a. ", "Z) "
//! - Roman numeral markers: "i. ", "III) "

// TODO: Move sequence marker logic from lexer.rs here
// This will be implemented in the next commit to keep changes atomic
