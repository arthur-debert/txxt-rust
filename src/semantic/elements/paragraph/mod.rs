//! Paragraph Element Construction
//!
//! Converts high-level tokens into paragraph AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/paragraph/`
//! - **AST Node**: `src/ast/elements/paragraph/block.rs`

use crate::ast::elements::paragraph::block::ParagraphBlock;
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::BlockParseError;

/// Create a paragraph element from a PlainTextLine token
///
/// # Arguments
/// * `token` - The PlainTextLine token to convert
///
/// # Returns
/// * `Result<ParagraphBlock, BlockParseError>`
pub fn create_paragraph_element(token: &HighLevelToken) -> Result<ParagraphBlock, BlockParseError> {
    match token {
        HighLevelToken::PlainTextLine { content, .. } => {
            // Extract content text
            let content_text = match content.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // Create a simple TextTransform::Identity for the plain text content
            let text = crate::ast::elements::inlines::Text::simple(&content_text);
            let text_transform = crate::ast::elements::inlines::TextTransform::Identity(text);

            Ok(ParagraphBlock {
                content: vec![text_transform],
                annotations: Vec::new(),
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                tokens: ScannerTokenSequence::new(),
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected PlainTextLine token for paragraph".to_string(),
        )),
    }
}
