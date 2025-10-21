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
                content: AnnotationContent::Inline(vec![]),
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                annotations: Vec::new(),
                tokens: ScannerTokenSequence::new(),
                namespace: None,
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected Annotation token for annotation".to_string(),
        )),
    }
}
