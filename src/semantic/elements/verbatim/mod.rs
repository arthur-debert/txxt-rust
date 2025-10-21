//! Verbatim Element Construction
//!
//! Converts high-level tokens into verbatim block AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/verbatim/`
//! - **AST Node**: `src/ast/elements/verbatim/block.rs`

use crate::ast::elements::verbatim::block::{VerbatimBlock, VerbatimType};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
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
        HighLevelToken::VerbatimBlock { title, label, .. } => {
            // Extract title text
            let _title_text = match title.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // Extract label text
            let label_text = match label.as_ref() {
                HighLevelToken::Label { text, .. } => text.clone(),
                _ => "unknown".to_string(),
            };

            Ok(VerbatimBlock {
                title: vec![], // TODO: Convert title_text to TextTransform
                content: crate::ast::elements::verbatim::ignore_container::IgnoreContainer::new(
                    vec![],
                    vec![],
                    vec![],
                    crate::ast::elements::components::parameters::Parameters::new(),
                    ScannerTokenSequence::new(),
                ),
                label: label_text,
                verbatim_type: VerbatimType::InFlow,
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                annotations: Vec::new(),
                tokens: ScannerTokenSequence::new(),
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected VerbatimBlock token for verbatim".to_string(),
        )),
    }
}
