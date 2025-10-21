//! Annotation Element Construction
//!
//! Converts high-level tokens into annotation AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/annotation/`
//! - **AST Node**: `src/ast/elements/annotation/annotation_block.rs`

use crate::ast::elements::annotation::annotation_block::{AnnotationBlock, AnnotationContent};
use crate::ast::elements::containers::content::ContentContainerElement;
use crate::ast::elements::containers::ContentContainer;
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::ast_construction::AstNode;
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
        HighLevelToken::Annotation { label, .. } => {
            // Extract label text
            let label_text = match label.as_ref() {
                HighLevelToken::Label { text, .. } => text.clone(),
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // Separate nested annotations from other content
            let mut nested_annotations = Vec::new();
            let mut other_content = Vec::new();

            for node in content_nodes {
                match node {
                    AstNode::Annotation(block) => nested_annotations.push(block.clone().into()),
                    _ => match node.to_element_node().try_into() {
                        Ok(element) => other_content.push(element),
                        Err(e) => {
                            return Err(BlockParseError::InvalidStructure(format!(
                                "Failed to convert nested element in annotation: {}",
                                e
                            )));
                        }
                    },
                }
            }

            // Determine content type
            let content = if other_content.is_empty() {
                AnnotationContent::Inline(vec![])
            } else {
                AnnotationContent::Block(ContentContainer::new(
                    other_content,
                    Vec::new(),
                    Default::default(),
                    Default::default(),
                ))
            };

            Ok(AnnotationBlock {
                label: label_text,
                content,
                annotations: nested_annotations,
                // FIXME: post-parser - Extract parameters from token instead of empty Parameters
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                // FIXME: post-parser - Preserve actual source tokens instead of empty sequence
                tokens: ScannerTokenSequence::new(),
                // FIXME: post-parser - Extract namespace from label (e.g., "org.example.custom")
                namespace: None,
            })
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
