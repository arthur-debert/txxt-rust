//! Verbatim Element Construction
//!
//! Converts high-level tokens into verbatim block AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/verbatim/`
//! - **AST Node**: `src/ast/elements/verbatim/block.rs`

use crate::ast::elements::verbatim::block::{VerbatimBlock, VerbatimType};
use crate::cst::{HighLevelToken, ScannerTokenSequence, WallType};
use crate::semantic::BlockParseError;

/// Create a verbatim block element from a VerbatimBlock token
///
/// # Arguments
/// * `token` - The VerbatimBlock token to convert
///
/// # Returns
/// * `Result<VerbatimBlock, BlockParseError>`
pub fn create_verbatim_element(token: &HighLevelToken) -> Result<VerbatimBlock, BlockParseError> {
    match token {
        HighLevelToken::VerbatimBlock {
            title,
            content,
            label,
            wall_type,
            ..
        } => {
            // Extract title text and convert to TextTransform
            let title_text = match title.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // FIXME: post-parser - Parse inline formatting in title instead of using Text::simple
            let title_transforms = if title_text.is_empty() {
                vec![]
            } else {
                vec![
                    crate::ast::elements::formatting::inlines::TextTransform::Identity(
                        crate::ast::elements::formatting::inlines::Text::simple(&title_text),
                    ),
                ]
            };

            // Extract content text (verbatim content is already extracted by scanner)
            let content_text = match content.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => String::new(),
            };

            // Extract label text
            let label_text = match label.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                HighLevelToken::Label { text, .. } => text.clone(),
                _ => "unknown".to_string(),
            };

            // Create IgnoreLine from the verbatim content
            let ignore_lines = if content_text.is_empty() {
                vec![]
            } else {
                vec![
                    crate::ast::elements::verbatim::ignore_container::IgnoreLine {
                        content: content_text,
                        // FIXME: post-parser - Preserve actual source tokens for ignore line
                        tokens: ScannerTokenSequence::new(),
                    },
                ]
            };

            // Create IgnoreContainer with the verbatim content
            let ignore_container =
                crate::ast::elements::verbatim::ignore_container::IgnoreContainer::new(
                    ignore_lines,
                    // FIXME: post-parser - Parse container-level comments
                    vec![],
                    // FIXME: post-parser - Parse container-level annotations
                    vec![],
                    // FIXME: post-parser - Extract parameters from verbatim block
                    crate::ast::elements::components::parameters::Parameters::new(),
                    // FIXME: post-parser - Preserve actual source tokens for container
                    ScannerTokenSequence::new(),
                );

            // Convert WallType to VerbatimType
            let verbatim_type = match wall_type {
                WallType::InFlow(_) => VerbatimType::InFlow,
                WallType::Stretched => VerbatimType::Stretched,
            };

            Ok(VerbatimBlock {
                title: title_transforms,
                content: ignore_container,
                label: label_text,
                verbatim_type,
                // FIXME: post-parser - Extract parameters from verbatim block token
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                // FIXME: post-parser - Parse block-level annotations
                annotations: Vec::new(),
                // FIXME: post-parser - Preserve actual source tokens for entire verbatim block
                tokens: ScannerTokenSequence::new(),
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected VerbatimBlock token for verbatim".to_string(),
        )),
    }
}
