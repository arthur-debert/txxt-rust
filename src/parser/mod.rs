//! TXXT document parser package
//!
//! This package provides modular parsing functionality for converting TokenBlocks
//! into Abstract Syntax Trees (AST). The parser uses a bottom-up approach to properly
//! handle session detection and container associations.

mod container_association;
mod document_parser;
mod element_parsers;
mod list_processing;
mod session_detection;
mod text_extraction;
mod token_analysis;

// Re-export the main entry point
pub use document_parser::{parse_document, DocumentParser};
