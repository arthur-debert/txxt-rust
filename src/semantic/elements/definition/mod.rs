//! Definition Element Construction
//!
//! Converts high-level tokens into definition AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/definition/`
//! - **AST Node**: `src/ast/elements/definition/block.rs`

use crate::ast::elements::definition::block::{DefinitionBlock, DefinitionTerm};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::elements::parameters::create_parameters_ast;
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
            term,
            parameters,
            tokens,
            ..
        } => {
            // Extract term text and source tokens
            let (term_text, source_tokens) = match term.as_ref() {
                HighLevelToken::TextSpan {
                    content, tokens, ..
                } => (content.clone(), tokens.clone()),
                _ => {
                    return Err(BlockParseError::InvalidStructure(
                        "Definition term must be a TextSpan".to_string(),
                    ))
                }
            };

            // Extract parameters using unified constructor
            // See: crate::semantic::elements::parameters::create_parameters_ast for single source of truth
            let extracted_params = create_parameters_ast(parameters.as_deref())?;

            // Create text transform for the term, preserving source tokens
            let term_content = vec![crate::ast::elements::inlines::TextTransform::Identity(
                crate::ast::elements::inlines::Text::simple_with_tokens(&term_text, source_tokens),
            )];

            // Convert content nodes to SimpleBlockElements
            // Per simple-container.txxt: Definitions can only contain Paragraph, List, Verbatim
            let mut simple_elements = Vec::new();
            for node in content_nodes.iter() {
                match node {
                    crate::semantic::ast_construction::AstNode::Paragraph(p) => {
                        simple_elements.push(
                            crate::ast::elements::containers::simple::SimpleBlockElement::Paragraph(
                                p.clone(),
                            ),
                        );
                    }
                    crate::semantic::ast_construction::AstNode::List(l) => {
                        simple_elements.push(
                            crate::ast::elements::containers::simple::SimpleBlockElement::List(
                                l.clone(),
                            ),
                        );
                    }
                    crate::semantic::ast_construction::AstNode::Verbatim(v) => {
                        simple_elements.push(
                            crate::ast::elements::containers::simple::SimpleBlockElement::Verbatim(
                                v.clone(),
                            ),
                        );
                    }
                    crate::semantic::ast_construction::AstNode::Definition(_) => {
                        return Err(BlockParseError::InvalidStructure(
                            "Cannot nest Definition inside Definition (SimpleContainer constraint)"
                                .to_string(),
                        ));
                    }
                    crate::semantic::ast_construction::AstNode::Annotation(_) => {
                        return Err(BlockParseError::InvalidStructure(
                            "Cannot nest Annotation inside Definition (SimpleContainer constraint)"
                                .to_string(),
                        ));
                    }
                    crate::semantic::ast_construction::AstNode::Session(_) => {
                        return Err(BlockParseError::InvalidStructure(
                            "Cannot nest Session inside Definition (SimpleContainer constraint)"
                                .to_string(),
                        ));
                    }
                }
            }

            // Create SimpleContainer with the parsed content
            let content_container = crate::ast::elements::containers::simple::SimpleContainer::new(
                simple_elements,
                // FIXME: post-parser - Parse container-level annotations instead of empty vec
                vec![],
                crate::ast::elements::components::parameters::Parameters::new(),
                ScannerTokenSequence::new(),
            );

            Ok(DefinitionBlock {
                term: DefinitionTerm {
                    content: term_content,
                    tokens: ScannerTokenSequence::new(),
                },
                content: content_container,
                parameters: extracted_params,
                // FIXME: post-parser - Parse definition-level annotations
                annotations: Vec::new(),
                tokens: tokens.clone(),
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected Definition token for definition".to_string(),
        )),
    }
}
