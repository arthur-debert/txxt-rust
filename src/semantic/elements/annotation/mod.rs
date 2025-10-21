//! Annotation Element Construction
//!
//! Converts high-level tokens into annotation AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/annotation/`
//! - **AST Node**: `src/ast/elements/annotation/annotation_block.rs`

use crate::ast::elements::annotation::annotation_block::{AnnotationBlock, AnnotationContent};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::BlockParseError;

/// Create an annotation element from an Annotation token
///
/// # Arguments
/// * `token` - The Annotation token to convert
///
/// # Returns
/// * `Result<AnnotationBlock, BlockParseError>`
pub fn create_annotation_element(
    token: &HighLevelToken,
) -> Result<AnnotationBlock, BlockParseError> {
    match token {
        HighLevelToken::Annotation { label, content, .. } => {
            // Extract label text
            let label_text = match label.as_ref() {
                HighLevelToken::Label { text, .. } => text.clone(),
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // Extract content text if present
            let _content_text = match content {
                Some(content_token) => match content_token.as_ref() {
                    HighLevelToken::TextSpan { content, .. } => Some(content.clone()),
                    HighLevelToken::PlainTextLine { content, .. } => match content.as_ref() {
                        HighLevelToken::TextSpan { content, .. } => Some(content.clone()),
                        _ => None,
                    },
                    _ => None,
                },
                None => None,
            };

            Ok(AnnotationBlock {
                label: label_text,
                // FIXME: post-parser - Parse inline content from _content_text instead of empty vec
                content: AnnotationContent::Inline(vec![]),
                // FIXME: post-parser - Extract parameters from token instead of empty Parameters
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                // FIXME: post-parser - Support nested annotations instead of empty vec
                annotations: Vec::new(),
                // FIXME: post-parser - Preserve actual source tokens instead of empty sequence
                tokens: ScannerTokenSequence::new(),
                // FIXME: post-parser - Extract namespace from label (e.g., "org.example.custom")
                namespace: None,
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected Annotation token for annotation".to_string(),
        )),
    }
}
