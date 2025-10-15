//! Session Block Element
//!
//! Session blocks - hierarchical sections of documents.

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    tokens::TokenSequence,
};

use super::super::{
    core::{BlockElement, ElementType, TxxtElement},
    inlines::TextTransform,
    list::{NumberingForm, NumberingStyle},
};

use super::session_container::SessionContainer;

/// Session block - a hierarchical section of the document
///
/// TODO: Migrate existing session logic from src/ast/structure.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionBlock {
    /// Session title with optional numbering
    pub title: SessionTitle,

    /// Session content (can contain nested sessions)
    pub content: SessionContainer,

    /// Annotations attached to this session
    pub annotations: Vec<Annotation>,

    /// Parameters for this session
    pub parameters: Parameters,

    /// Raw tokens for source reconstruction
    pub tokens: TokenSequence,
}

/// Session title with hierarchical numbering support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTitle {
    /// The title content with token-level precision
    pub content: Vec<TextTransform>,

    /// Optional session numbering (e.g., "1.2.3", "a)", "i.")
    pub numbering: Option<SessionNumbering>,

    /// Raw tokens for exact source reconstruction
    pub tokens: TokenSequence,
}

/// Session numbering information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionNumbering {
    /// The actual numbering text as it appears in source
    pub marker: String,

    /// Detected numbering style for this session level
    pub style: NumberingStyle,

    /// Whether this is short form or full form
    pub form: NumberingForm,
}

impl TxxtElement for SessionBlock {
    fn element_type(&self) -> ElementType {
        ElementType::Block
    }

    fn tokens(&self) -> &TokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl BlockElement for SessionBlock {
    fn can_contain_blocks(&self) -> bool {
        true // Sessions can contain other blocks
    }

    fn content_summary(&self) -> String {
        format!("Session: {}", self.title.text_content())
    }
}

impl TxxtElement for SessionTitle {
    fn element_type(&self) -> ElementType {
        ElementType::Line // Session titles are line elements
    }

    fn tokens(&self) -> &TokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &[] // Session titles don't have direct annotations
    }

    fn parameters(&self) -> &Parameters {
        // Session titles don't have parameters - use a static empty instance
        use std::sync::OnceLock;
        static EMPTY_PARAMS: OnceLock<Parameters> = OnceLock::new();
        EMPTY_PARAMS.get_or_init(Parameters::default)
    }
}

impl SessionBlock {
    /// Create a new session block
    pub fn new(
        title: SessionTitle,
        content: SessionContainer,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: TokenSequence,
    ) -> Self {
        Self {
            title,
            content,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Get the session title text
    pub fn title_text(&self) -> String {
        self.title.text_content()
    }

    /// Check if this session has numbering
    pub fn has_numbering(&self) -> bool {
        self.title.numbering.is_some()
    }

    /// Get the numbering marker if present
    pub fn numbering_marker(&self) -> Option<&str> {
        self.title.numbering.as_ref().map(|n| n.marker.as_str())
    }

    /// Check if the session content is empty
    pub fn is_content_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl SessionTitle {
    /// Create a new session title
    pub fn new(
        content: Vec<TextTransform>,
        numbering: Option<SessionNumbering>,
        tokens: TokenSequence,
    ) -> Self {
        Self {
            content,
            numbering,
            tokens,
        }
    }

    /// Get the text content of the title
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl SessionNumbering {
    /// Create a new session numbering
    pub fn new(marker: String, style: NumberingStyle, form: NumberingForm) -> Self {
        Self {
            marker,
            style,
            form,
        }
    }

    /// Check if this uses short form numbering
    pub fn is_short_form(&self) -> bool {
        matches!(self.form, NumberingForm::Short)
    }

    /// Check if this uses full form numbering
    pub fn is_full_form(&self) -> bool {
        matches!(self.form, NumberingForm::Full)
    }
}
