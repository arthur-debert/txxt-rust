//! Session Element Construction
//!
//! Converts high-level tokens into session AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/session/`
//! - **AST Node**: `src/ast/elements/session/block.rs`

use crate::ast::elements::inlines::TextTransform;
use crate::ast::elements::session::block::{SessionBlock, SessionTitle};
use crate::ast::elements::session::session_container::{SessionContainer, SessionContainerElement};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::ast_construction::AstNode;
use crate::semantic::BlockParseError;

/// Create a session element from parsed components
///
/// Sessions are complex structures that require multiple tokens to construct.
/// This function takes the title token and child nodes (already parsed).
///
/// # Arguments
/// * `title_token` - The token containing the session title
/// * `_child_nodes` - The parsed child nodes (content of the session)
///
/// # Returns
/// * `Result<SessionBlock, BlockParseError>`
pub fn create_session_element(
    title_token: &HighLevelToken,
    child_nodes: &[AstNode],
) -> Result<SessionBlock, BlockParseError> {
    // Extract title text from the title token
    let title_text = match title_token {
        HighLevelToken::PlainTextLine { content, .. } => match content.as_ref() {
            HighLevelToken::TextSpan { content, .. } => content.clone(),
            _ => "unknown".to_string(),
        },
        HighLevelToken::SequenceTextLine { content, .. } => match content.as_ref() {
            HighLevelToken::TextSpan { content, .. } => content.clone(),
            _ => "unknown".to_string(),
        },
        HighLevelToken::Definition { term, .. } => match term.as_ref() {
            HighLevelToken::TextSpan { content, .. } => content.clone(),
            _ => "unknown".to_string(),
        },
        _ => {
            return Err(BlockParseError::InvalidStructure(
                "Invalid title token for session".to_string(),
            ))
        }
    };

    // Convert title text to TextTransform using Text::simple helper
    let title_content = if !title_text.is_empty() {
        let text = crate::ast::elements::inlines::Text::simple(&title_text);
        vec![TextTransform::Identity(text)]
    } else {
        vec![]
    };

    // Convert AstNodes to SessionContainerElements
    let content_elements: Vec<SessionContainerElement> = child_nodes
        .iter()
        .map(|node| match node {
            AstNode::Paragraph(p) => SessionContainerElement::Paragraph(p.clone()),
            AstNode::Session(s) => SessionContainerElement::Session(s.clone()),
            AstNode::List(l) => SessionContainerElement::List(l.clone()),
            AstNode::Definition(d) => SessionContainerElement::Definition(d.clone()),
        })
        .collect();

    Ok(SessionBlock {
        title: SessionTitle {
            content: title_content,
            numbering: None,
            tokens: ScannerTokenSequence::new(),
        },
        content: SessionContainer {
            content: content_elements,
            annotations: Vec::new(),
            parameters: crate::ast::elements::components::parameters::Parameters::new(),
            tokens: ScannerTokenSequence::new(),
        },
        annotations: Vec::new(),
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: ScannerTokenSequence::new(),
    })
}
