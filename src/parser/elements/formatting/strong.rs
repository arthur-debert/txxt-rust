//! # Strong (Bold) Element Parser
//!
//! This module implements the parsing logic for strong (bold) formatting elements
//! using the `*content*` pattern.
//!
//! ## Overview
//!
//! Strong elements provide visual and semantic emphasis for important content.
//! They follow the general inline token pattern with single asterisk tokens
//! and support nested formatting (except other strong elements).
//!
//! ## Syntax
//!
//! - **Pattern**: `*content*`
//! - **Token**: Single asterisk (`*`)
//! - **Purpose**: Strong importance, key concepts, warnings
//! - **Semantic meaning**: High-priority information
//! - **Visual rendering**: Bold text
//! - **Nesting**: Can contain other inline types (except strong)
//!
//! ## Grammar
//!
//! From [`docs/specs/core/syntax.txxt`]:
//! ```text
//! <bold-span> = <asterisk> <text-content> <asterisk>
//! ```
//!
//! ## Processing Rules
//!
//! ### Recognition Criteria
//! - Starts and ends with single asterisk (`*`)
//! - No spaces between asterisk and content boundaries
//! - Content cannot be empty
//! - Content cannot span line breaks
//! - Cannot nest within other strong elements
//!
//! ### Content Processing
//! - Allows nested inline elements (recursive parsing)
//! - Content is processed for other formatting types
//! - Maintains token-level precision for language server support
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/formatting/formatting.txxt`]
//! - **AST Node**: [`src/ast/elements/formatting/`]
//! - **Tokenizer**: [`src/lexer/elements/formatting/delimiters.rs`]

use crate::ast::elements::formatting::inlines::{Text, TextTransform};
use crate::cst::ScannerToken;
use crate::parser::elements::inlines::InlineParseError;

/// Parse a strong (bold) formatting element from tokens
///
/// Handles strong formatting using the `*content*` pattern.
/// Supports nested formatting within the content.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing strong content
///
/// # Returns
/// * `Result<TextTransform, InlineParseError>`
///
/// # Examples
///
/// ```rust,ignore
/// // Simple strong text
/// let tokens = tokenize("*important*");
/// let strong = parse_strong(&tokens)?;
///
/// // Strong with nested formatting
/// let tokens = tokenize("*_bold italic_*");
/// let strong = parse_strong(&tokens)?;
/// ```
pub fn parse_strong(tokens: &[ScannerToken]) -> Result<TextTransform, InlineParseError> {
    if tokens.is_empty() {
        return Err(InlineParseError::InvalidStructure(
            "Empty strong tokens".to_string(),
        ));
    }

    // Check if this looks like a strong pattern and extract content
    if !is_strong_pattern(tokens) {
        return Err(InlineParseError::InvalidStructure(
            "Tokens do not match strong pattern".to_string(),
        ));
    }

    // Extract content between the asterisks
    let content_tokens = extract_strong_content(tokens)?;

    // Validate nesting rules
    validate_strong_nesting(&content_tokens)?;

    // Convert content tokens to text (for now, simple implementation)
    let text_content = content_tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    if text_content.is_empty() {
        return Err(InlineParseError::EmptyContent(
            "Empty strong content".to_string(),
        ));
    }

    // Create a strong transform with identity content
    let content_transform = TextTransform::Identity(Text::simple(&text_content));
    let strong_transform = TextTransform::Strong(vec![content_transform]);

    Ok(strong_transform)
}

/// Check if tokens represent a valid strong pattern
///
/// Validates that the token sequence follows the `*content*` pattern
/// required for strong formatting.
///
/// # Arguments
/// * `tokens` - Sequence of tokens to validate
///
/// # Returns
/// * `bool` - True if tokens represent valid strong pattern
pub fn is_strong_pattern(tokens: &[ScannerToken]) -> bool {
    // TODO: Implement proper strong pattern detection
    // For now, return a simple check

    if tokens.len() < 3 {
        return false;
    }

    // Very basic pattern check - should be enhanced
    matches!(tokens.first(), Some(ScannerToken::Text { content, .. }) if content == "*")
        && matches!(tokens.last(), Some(ScannerToken::Text { content, .. }) if content == "*")
}

/// Extract content tokens from a strong pattern
///
/// Extracts the content between the asterisk delimiters for further processing.
///
/// # Arguments
/// * `tokens` - Sequence of tokens in strong pattern
///
/// # Returns
/// * `Result<Vec<ScannerToken>, InlineParseError>`
pub fn extract_strong_content(
    tokens: &[ScannerToken],
) -> Result<Vec<ScannerToken>, InlineParseError> {
    if tokens.len() < 3 {
        return Err(InlineParseError::InvalidStructure(
            "Strong pattern requires at least 3 tokens".to_string(),
        ));
    }

    // TODO: Implement proper content extraction
    // For now, return all tokens except first and last (the asterisks)

    let content_tokens = tokens[1..tokens.len() - 1].to_vec();

    if content_tokens.is_empty() {
        return Err(InlineParseError::EmptyContent(
            "Strong content cannot be empty".to_string(),
        ));
    }

    Ok(content_tokens)
}

/// Validate strong content for nesting rules
///
/// Ensures that strong content doesn't violate nesting rules
/// (e.g., no strong within strong).
///
/// # Arguments
/// * `content_tokens` - Tokens within the strong element
///
/// # Returns
/// * `Result<(), InlineParseError>`
pub fn validate_strong_nesting(content_tokens: &[ScannerToken]) -> Result<(), InlineParseError> {
    // TODO: Implement proper nesting validation
    // For now, accept all content

    // Check for nested asterisks that would indicate invalid nesting
    for token in content_tokens {
        if let ScannerToken::Text { content, .. } = token {
            if content == "*" {
                return Err(InlineParseError::InvalidNesting(
                    "Strong elements cannot be nested within other strong elements".to_string(),
                ));
            }
        }
    }

    Ok(())
}
