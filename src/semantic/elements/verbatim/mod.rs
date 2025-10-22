//! Verbatim Element Construction
//!
//! Converts high-level tokens into verbatim block AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/verbatim/`
//! - **AST Node**: `src/ast/elements/verbatim/block.rs`

use crate::ast::elements::verbatim::block::{VerbatimBlock, VerbatimType};
use crate::cst::{HighLevelToken, ScannerTokenSequence, WallType};
use crate::semantic::elements::parameters::create_parameters_ast;
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
        HighLevelToken::VerbatimBlock {
            title,
            content,
            label,
            parameters,
            wall_type,
            ..
        } => {
            // Extract title text and source tokens, convert to TextTransform
            let (title_text, title_tokens) = match title.as_ref() {
                HighLevelToken::TextSpan {
                    content, tokens, ..
                } => (content.clone(), tokens.clone()),
                _ => {
                    return Err(BlockParseError::InvalidStructure(
                        "Verbatim title must be a TextSpan".to_string(),
                    ))
                }
            };

            // FIXME: post-parser - Parse inline formatting in title instead of using Text::simple
            let title_transforms = if title_text.is_empty() {
                vec![]
            } else {
                vec![
                    crate::ast::elements::formatting::inlines::TextTransform::Identity(
                        crate::ast::elements::formatting::inlines::Text::simple_with_tokens(
                            &title_text,
                            title_tokens,
                        ),
                    ),
                ]
            };

            // Extract label text
            let label_text = match label.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                HighLevelToken::Label { text, .. } => text.clone(),
                _ => "unknown".to_string(),
            };

            // Determine wall indentation level for stripping
            let wall_indent = match wall_type {
                WallType::InFlow(indent) => indent + 4, // Content at indent + 4
                WallType::Stretched => 0,               // No wall stripping for stretched
            };

            // Create AST IgnoreLine nodes from high-level IgnoreLine/BlankLine tokens
            // Apply wall-stripping based on wall_type
            let mut ignore_lines = Vec::new();
            for high_level_token in content {
                match high_level_token {
                    HighLevelToken::IgnoreLine {
                        content: line_content,
                        tokens,
                        ..
                    } => {
                        // Wall-stripping: remove wall indentation from IgnoreLine content
                        let stripped_content = if *wall_type == WallType::Stretched {
                            // Stretched: keep everything as-is
                            line_content.clone()
                        } else {
                            // InFlow: strip wall indentation
                            if line_content.len() >= wall_indent {
                                // Strip the wall indent
                                line_content[wall_indent..].to_string()
                            } else {
                                // Line has less indentation than wall - keep as-is
                                line_content.clone()
                            }
                        };

                        ignore_lines.push(
                            crate::ast::elements::verbatim::ignore_container::IgnoreLine {
                                content: stripped_content,
                                tokens: tokens.clone(),
                            },
                        );
                    }
                    HighLevelToken::BlankLine { tokens, .. } => {
                        // Preserve blank lines
                        ignore_lines.push(
                            crate::ast::elements::verbatim::ignore_container::IgnoreLine {
                                content: String::new(),
                                tokens: tokens.clone(),
                            },
                        );
                    }
                    _ => {
                        // Unexpected token type in content - skip it
                    }
                }
            }

            // Extract parameters using unified constructor
            let extracted_params = create_parameters_ast(parameters.as_deref())?;

            // Create IgnoreContainer with the verbatim content
            let ignore_container =
                crate::ast::elements::verbatim::ignore_container::IgnoreContainer::new(
                    ignore_lines,
                    // FIXME: post-parser - Parse container-level comments
                    vec![],
                    // FIXME: post-parser - Parse container-level annotations
                    vec![],
                    extracted_params.clone(),
                    ScannerTokenSequence::new(),
                );

            // Convert WallType to VerbatimType
            let verbatim_type = match wall_type {
                WallType::InFlow(_) => VerbatimType::InFlow,
                WallType::Stretched => VerbatimType::Stretched,
            };

            Ok(VerbatimBlock {
                title: title_transforms,
                content: ignore_container,
                label: label_text,
                verbatim_type,
                // Parameters extracted using unified constructor
                parameters: extracted_params,
                // FIXME: post-parser - Parse block-level annotations
                annotations: Vec::new(),
                tokens: ScannerTokenSequence::new(),
            })
        }
        _ => Err(BlockParseError::InvalidStructure(
            "Expected VerbatimBlock token for verbatim".to_string(),
        )),
    }
}
