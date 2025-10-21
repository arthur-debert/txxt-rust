//! Definition Element Construction
//!
//! Converts high-level tokens into definition AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/definition/`
//! - **AST Node**: `src/ast/elements/definition/block.rs`

use crate::ast::elements::definition::block::{DefinitionBlock, DefinitionTerm};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::BlockParseError;

/// Create a definition element from a Definition token and content nodes
///
/// # Arguments
/// * `token` - The Definition token containing the term
/// * `content_nodes` - The parsed AST nodes for the definition content
///
/// # Returns
/// * `Result<DefinitionBlock, BlockParseError>`
pub fn create_definition_element(
    token: &HighLevelToken,
    content_nodes: &[crate::semantic::ast_construction::AstNode],
) -> Result<DefinitionBlock, BlockParseError> {
    match token {
        HighLevelToken::Definition {
            term, parameters, ..
        } => {
            // Extract term text
            let term_text = match term.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // Create simple text transform for the term
            let term_content = vec![crate::ast::elements::inlines::TextTransform::Identity(
                crate::ast::elements::inlines::Text::simple(&term_text),
            )];

            // Convert content nodes to ContentContainerElements
            let content_elements: Vec<crate::ast::elements::containers::content::ContentContainerElement> = content_nodes
                .iter()
                .map(|node| match node {
                    crate::semantic::ast_construction::AstNode::Paragraph(p) => {
                        crate::ast::elements::containers::content::ContentContainerElement::Paragraph(p.clone())
                    }
                    crate::semantic::ast_construction::AstNode::List(l) => {
                        crate::ast::elements::containers::content::ContentContainerElement::List(l.clone())
                    }
                    crate::semantic::ast_construction::AstNode::Definition(d) => {
                        crate::ast::elements::containers::content::ContentContainerElement::Definition(d.clone())
                    }
                    crate::semantic::ast_construction::AstNode::Session(_) => {
                        // Sessions cannot be in ContentContainer - this shouldn't happen
                        panic!("Sessions cannot be inside definitions (ContentContainer restriction)")
                    }
                })
                .collect();

            // Create ContentContainer with the parsed content
            let content_container =
                crate::ast::elements::containers::content::ContentContainer::new(
                    content_elements,
                    vec![], // No annotations for now
                    crate::ast::elements::components::parameters::Parameters::new(),
                    ScannerTokenSequence::new(),
                );

            // Handle parameters if present (future extension)
            let _ = parameters;

            Ok(DefinitionBlock {
                term: DefinitionTerm {
                    content: term_content,
                    tokens: ScannerTokenSequence::new(),
                },
                content: content_container,
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                annotations: Vec::new(),
                tokens: ScannerTokenSequence::new(),
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected Definition token for definition".to_string(),
        )),
    }
}
