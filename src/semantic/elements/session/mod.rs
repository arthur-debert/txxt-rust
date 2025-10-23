//! Session Element Construction
//!
//! Converts high-level tokens into session AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/session/`
//! - **AST Node**: `src/ast/elements/session/block.rs`

use crate::ast::elements::inlines::TextTransform;
use crate::ast::elements::session::block::{SessionBlock, SessionNumbering, SessionTitle};
use crate::ast::elements::session::session_container::{SessionContainer, SessionContainerElement};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::ast_construction::AstNode;
use crate::semantic::BlockParseError;

/// Extract numbering information from a sequence marker
///
/// Sessions can have hierarchical numbering like "1.2.3" which appears as a
/// SequenceMarker in the high-level token stream. This function extracts the
/// marker string and determines its style and form using shared numbering utilities.
///
/// Plain markers ("-") are NOT valid for sessions - only numerical, alphabetical,
/// and roman numeral markers are supported.
fn extract_numbering_from_marker(
    marker_token: &HighLevelToken,
) -> Result<Option<SessionNumbering>, BlockParseError> {
    // Extract marker string from marker token (same as lists do)
    let marker = match marker_token {
        HighLevelToken::SequenceMarker { marker, .. } => marker.clone(),
        HighLevelToken::TextSpan { content, .. } => content.clone(),
        _ => return Ok(None),
    };

    // Use list_detection to determine marker type
    use crate::syntax::list_detection;
    let decoration = list_detection::determine_decoration_type(&marker);

    // Convert to AST types using shared utilities
    // Sessions do NOT allow plain markers (allow_plain = false)
    use crate::semantic::elements::numbering::{convert_numbering_form, convert_numbering_style};

    let style = match convert_numbering_style(&decoration.style, false) {
        Some(s) => s,
        None => return Ok(None), // Plain markers not allowed for sessions
    };

    let form = convert_numbering_form(&decoration.form);

    Ok(Some(SessionNumbering {
        marker,
        style,
        form,
    }))
}

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
    // Extract title text, tokens, and numbering from the title token
    let (title_text, source_tokens, numbering) = match title_token {
        HighLevelToken::PlainTextLine { content, .. } => match content.as_ref() {
            HighLevelToken::TextSpan {
                content, tokens, ..
            } => (content.clone(), tokens.clone(), None),
            _ => {
                return Err(BlockParseError::InvalidStructure(
                    "PlainTextLine content must be a TextSpan".to_string(),
                ))
            }
        },
        HighLevelToken::SequenceTextLine {
            marker, content, ..
        } => match content.as_ref() {
            HighLevelToken::TextSpan {
                content, tokens, ..
            } => {
                // Extract numbering from marker
                let numbering = extract_numbering_from_marker(marker)?;
                (content.clone(), tokens.clone(), numbering)
            }
            _ => {
                return Err(BlockParseError::InvalidStructure(
                    "SequenceTextLine content must be a TextSpan".to_string(),
                ))
            }
        },
        HighLevelToken::Definition { term, .. } => match term.as_ref() {
            HighLevelToken::TextSpan {
                content, tokens, ..
            } => (content.clone(), tokens.clone(), None),
            _ => {
                return Err(BlockParseError::InvalidStructure(
                    "Definition term must be a TextSpan".to_string(),
                ))
            }
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
            numbering,
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
