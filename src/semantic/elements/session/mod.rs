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
    // Extract title text and tokens from the title token
    let (title_text, source_tokens) = match title_token {
        HighLevelToken::PlainTextLine { content, .. } => match content.as_ref() {
            HighLevelToken::TextSpan {
                content, tokens, ..
            } => (content.clone(), tokens.clone()),
            _ => ("unknown".to_string(), None),
        },
        HighLevelToken::SequenceTextLine { content, .. } => match content.as_ref() {
            HighLevelToken::TextSpan {
                content, tokens, ..
            } => (content.clone(), tokens.clone()),
            _ => ("unknown".to_string(), None),
        },
        HighLevelToken::Definition { term, .. } => match term.as_ref() {
            HighLevelToken::TextSpan {
                content, tokens, ..
            } => (content.clone(), tokens.clone()),
            _ => ("unknown".to_string(), None),
        },
        _ => {
            return Err(BlockParseError::InvalidStructure(
                "Invalid title token for session".to_string(),
            ))
        }
    };

    // Convert title text to TextTransform, preserving source tokens
    let title_content = if !title_text.is_empty() {
        let text =
            crate::ast::elements::inlines::Text::simple_with_tokens(&title_text, source_tokens);
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
            AstNode::Annotation(a) => SessionContainerElement::Annotation(a.clone()),
            AstNode::Verbatim(v) => SessionContainerElement::Verbatim(v.clone()),
        })
        .collect();

    Ok(SessionBlock {
        title: SessionTitle {
            // FIXME: post-parser - Parse inline formatting in title instead of using Text::simple
            content: title_content,
            // FIXME: post-parser - Extract numbering from title_token (e.g., "1.2.3")
            numbering: None,
            tokens: ScannerTokenSequence::new(),
        },
        content: SessionContainer {
            content: content_elements,
            // FIXME: post-parser - Parse container-level annotations
            annotations: Vec::new(),
            // FIXME: post-parser - Extract parameters from session
            parameters: crate::ast::elements::components::parameters::Parameters::new(),
            tokens: ScannerTokenSequence::new(),
        },
        // FIXME: post-parser - Parse session-level annotations
        annotations: Vec::new(),
        // FIXME: post-parser - Extract parameters from session
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: ScannerTokenSequence::new(),
    })
}
