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

/// Create a definition element from a Definition token
///
/// # Arguments
/// * `token` - The Definition token to convert
///
/// # Returns
/// * `Result<DefinitionBlock, BlockParseError>`
pub fn create_definition_element(
    token: &HighLevelToken,
) -> Result<DefinitionBlock, BlockParseError> {
    match token {
        HighLevelToken::Definition {
            term, parameters, ..
        } => {
            // Extract term text
            let _term_text = match term.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            };

            // Extract parameters text if present
            let _params_text = match parameters {
                Some(params_token) => {
                    match params_token.as_ref() {
                        HighLevelToken::Parameters { params, .. } => {
                            // Convert parameters to string representation
                            let param_strings: Vec<String> =
                                params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                            Some(param_strings.join(","))
                        }
                        _ => None,
                    }
                }
                None => None,
            };

            Ok(DefinitionBlock {
                term: DefinitionTerm {
                    content: vec![], // TODO: Convert term_text to TextTransform
                    tokens: ScannerTokenSequence::new(),
                },
                content: crate::ast::elements::containers::content::ContentContainer::new(
                    vec![],
                    vec![],
                    crate::ast::elements::components::parameters::Parameters::new(),
                    ScannerTokenSequence::new(),
                ),
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
