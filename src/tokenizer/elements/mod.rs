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

// Re-export main interfaces for backward compatibility
pub use annotation::read_annotation_marker;
pub use components::{parse_label, validate_label, Label, LabelParseResult};
pub use components::{parse_parameters, ParameterLexer};
pub use containers::{
    detect_container_end, detect_container_start, determine_container_type,
    validate_container_content, ContainerContext, ContainerType,
};
pub use definition::read_definition_marker;
pub use list::read_sequence_marker;
pub use paragraph::{
    collect_paragraph_lines, detect_paragraph, should_terminate_paragraph, Paragraph,
    ParagraphParseResult,
};
pub use session::{
    confirm_session_with_content, detect_session_title, format_session_numbering,
    SessionParseResult, SessionTitle,
};
