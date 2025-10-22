//! Verbatim Element Construction
//!
//! Converts high-level tokens into verbatim block AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/verbatim/`
//! - **AST Node**: `src/ast/elements/verbatim/block.rs`

use crate::ast::elements::verbatim::block::{VerbatimBlock, VerbatimType};
use crate::cst::{HighLevelToken, ScannerToken, ScannerTokenSequence, WallType};
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

            // Extract line-level content from scanner tokens (Phase 4.3)
            // Get scanner tokens from the content high-level token
            let content_scanner_tokens = match content.as_ref() {
                HighLevelToken::TextSpan { tokens, .. } => &tokens.tokens,
                _ => &vec![],
            };

            // Extract label text
            let label_text = match label.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                HighLevelToken::Label { text, .. } => text.clone(),
                _ => "unknown".to_string(),
            };

            // Determine wall indentation level for stripping (Phase 4.4)
            let wall_indent = match wall_type {
                WallType::InFlow(indent) => indent + 4, // Content at indent + 4
                WallType::Stretched => 0,               // No wall stripping for stretched
            };

            // Create IgnoreLine from VerbatimContentLine scanner tokens (Phase 4.3 & 4.4)
            let mut ignore_lines = Vec::new();
            for scanner_token in content_scanner_tokens {
                match scanner_token {
                    ScannerToken::VerbatimContentLine {
                        indentation,
                        content: line_content,
                        ..
                    } => {
                        // Wall-stripping: remove wall indentation (Phase 4.4)
                        let stripped_content = if *wall_type == WallType::Stretched {
                            // Stretched: keep everything as-is
                            format!("{}{}", indentation, line_content)
                        } else {
                            // InFlow: strip wall indentation
                            let full_line = format!("{}{}", indentation, line_content);
                            if indentation.len() >= wall_indent {
                                // Strip the wall indent
                                full_line[wall_indent..].to_string()
                            } else {
                                // Line has less indentation than wall - keep as-is
                                full_line
                            }
                        };

                        ignore_lines.push(
                            crate::ast::elements::verbatim::ignore_container::IgnoreLine {
                                content: stripped_content,
                                tokens: ScannerTokenSequence {
                                    tokens: vec![scanner_token.clone()],
                                },
                            },
                        );
                    }
                    ScannerToken::BlankLine { .. } => {
                        // Preserve blank lines
                        ignore_lines.push(
                            crate::ast::elements::verbatim::ignore_container::IgnoreLine {
                                content: String::new(),
                                tokens: ScannerTokenSequence {
                                    tokens: vec![scanner_token.clone()],
                                },
                            },
                        );
                    }
                    _ => {}
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
