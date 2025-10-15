//! TXXT Tokenizer Elements - Organized by specification structure
//!
//! This module contains all tokenizer element implementations organized according
//! to the standard file layout described in docs/dev/file-layout.txxt.
//!
//! Each subdirectory corresponds to a specific element type and contains
//! the tokenization logic for that element.

pub mod annotation;
pub mod components;
pub mod containers;
pub mod definition;
pub mod document;
pub mod formatting;
pub mod list;
pub mod paragraph;
pub mod references;
pub mod session;
pub mod verbatim;
