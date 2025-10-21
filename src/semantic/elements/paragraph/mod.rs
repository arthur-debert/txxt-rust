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
    create_paragraph_element_multi(std::slice::from_ref(token))
}

/// Create a paragraph element from multiple PlainTextLine tokens
///
/// Paragraphs in txxt consist of consecutive PlainTextLine tokens terminated by
/// a blank line or other element.
///
/// # Arguments
/// * `tokens` - Vector of PlainTextLine tokens to combine into one paragraph
///
/// # Returns
/// * `Result<ParagraphBlock, BlockParseError>`
pub fn create_paragraph_element_multi(
    tokens: &[HighLevelToken],
) -> Result<ParagraphBlock, BlockParseError> {
    if tokens.is_empty() {
        return Err(BlockParseError::InvalidStructure(
            "Paragraph requires at least one line".to_string(),
        ));
    }

    let mut content_transforms = Vec::new();

    for token in tokens {
        match token {
            HighLevelToken::PlainTextLine { content, .. } => {
                // Extract content text and source tokens
                let (content_text, source_tokens) = match content.as_ref() {
                    HighLevelToken::TextSpan {
                        content, tokens, ..
                    } => (content.clone(), tokens.clone()),
                    _ => ("unknown".to_string(), None),
                };

                // Create TextTransform::Identity, preserving source tokens
                let text = crate::ast::elements::inlines::Text::simple_with_tokens(
                    &content_text,
                    source_tokens,
                );
                let text_transform = crate::ast::elements::inlines::TextTransform::Identity(text);
                content_transforms.push(text_transform);
            }
            _ => {
                return Err(BlockParseError::InvalidStructure(
                    "Expected PlainTextLine token for paragraph".to_string(),
                ))
            }
        }
    }

    Ok(ParagraphBlock {
        // FIXME: post-parser - Parse inline formatting in content instead of using Text::simple
        content: content_transforms,
        // FIXME: post-parser - Parse paragraph-level annotations
        annotations: Vec::new(),
        // FIXME: post-parser - Extract parameters from paragraph
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: ScannerTokenSequence::new(),
    })
}
