//! Annotation Element Construction
//!
//! Converts high-level tokens into annotation AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/annotation/`
//! - **AST Node**: `src/ast/elements/annotation/annotation_block.rs`

use crate::ast::elements::annotation::annotation_block::{AnnotationBlock, AnnotationContent};
use crate::ast::elements::containers::content::ContentContainerElement;
use crate::cst::HighLevelToken;
use crate::semantic::ast_construction::AstNode;
use crate::semantic::elements::parameters::create_parameters_ast;
use crate::semantic::BlockParseError;

/// Create an annotation element from an Annotation token and its parsed content
///
/// # Arguments
/// * `token` - The Annotation token to convert
/// * `content_nodes` - Parsed AST nodes from the annotation's indented content
///
/// # Returns
/// * `Result<AnnotationBlock, BlockParseError>`
pub fn create_annotation_element(
    token: &HighLevelToken,
    content_nodes: &[AstNode],
) -> Result<AnnotationBlock, BlockParseError> {
    match token {
        HighLevelToken::Annotation {
            label,
            parameters,
            tokens,
            ..
        } => {
            // Extract label text
            let label_text = match label.as_ref() {
                HighLevelToken::Label { text, .. } => text.clone(),
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // Extract parameters using unified constructor
            // See: crate::semantic::elements::parameters::create_parameters_ast for single source of truth
            let extracted_params = create_parameters_ast(parameters.as_deref())?;

            // Convert content nodes to SimpleBlockElements
            // Per simple-container.txxt: Annotations can only contain Paragraph, List, Verbatim
            // No nested annotations allowed (prevents unbounded recursion)
            let mut simple_elements = Vec::new();

            for node in content_nodes {
                match node {
                    AstNode::Paragraph(p) => {
                        simple_elements.push(
                            crate::ast::elements::containers::simple::SimpleBlockElement::Paragraph(
                                p.clone(),
                            ),
                        );
                    }
                    AstNode::List(l) => {
                        simple_elements.push(
                            crate::ast::elements::containers::simple::SimpleBlockElement::List(
                                l.clone(),
                            ),
                        );
                    }
                    AstNode::Verbatim(v) => {
                        simple_elements.push(
                            crate::ast::elements::containers::simple::SimpleBlockElement::Verbatim(
                                v.clone(),
                            ),
                        );
                    }
                    AstNode::Annotation(_) => {
                        return Err(BlockParseError::InvalidStructure(
                            "Cannot nest Annotation inside Annotation (SimpleContainer constraint)"
                                .to_string(),
                        ));
                    }
                    AstNode::Definition(_) => {
                        return Err(BlockParseError::InvalidStructure(
                            "Cannot nest Definition inside Annotation (SimpleContainer constraint)"
                                .to_string(),
                        ));
                    }
                    AstNode::Session(_) => {
                        return Err(BlockParseError::InvalidStructure(
                            "Cannot nest Session inside Annotation (SimpleContainer constraint)"
                                .to_string(),
                        ));
                    }
                }
            }

            // Determine content type
            let content = if simple_elements.is_empty() {
                AnnotationContent::Inline(vec![])
            } else {
                AnnotationContent::Block(
                    crate::ast::elements::containers::simple::SimpleContainer::new(
                        simple_elements,
                        Vec::new(),
                        Default::default(),
                        Default::default(),
                    ),
                )
            };

            Ok(AnnotationBlock::new(
                label_text,
                content,
                extracted_params,
                Vec::new(), // No nested annotations allowed in SimpleContainer
                tokens.clone(),
            ))
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected Annotation token for annotation".to_string(),
        )),
    }
}

impl TryFrom<crate::ast::elements::core::ElementNode> for ContentContainerElement {
    type Error = BlockParseError;

    fn try_from(node: crate::ast::elements::core::ElementNode) -> Result<Self, Self::Error> {
        match node {
            crate::ast::elements::core::ElementNode::ParagraphBlock(block) => {
                Ok(ContentContainerElement::Paragraph(block))
            }
            crate::ast::elements::core::ElementNode::ListBlock(block) => {
                Ok(ContentContainerElement::List(block))
            }
            crate::ast::elements::core::ElementNode::DefinitionBlock(block) => {
                Ok(ContentContainerElement::Definition(block))
            }
            crate::ast::elements::core::ElementNode::VerbatimBlock(block) => {
                Ok(ContentContainerElement::Verbatim(block))
            }
            crate::ast::elements::core::ElementNode::AnnotationBlock(block) => {
                Ok(ContentContainerElement::Annotation(block))
            }
            other => Err(BlockParseError::InvalidStructure(
                format!("Element type {:?} not allowed in ContentContainer (only Paragraph, List, Definition, Verbatim, Annotation are allowed)",
                    std::mem::discriminant(&other)),
            )),
        }
    }
}
