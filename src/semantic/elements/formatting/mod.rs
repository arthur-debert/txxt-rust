//! # Formatting Elements Parser Module
//!
//! This module contains the parsing logic for formatting inline elements - text
//! formatting elements that provide visual emphasis and semantic markup within txxt documents.
//!
//! ## Overview
//!
//! Formatting elements follow the general inline token pattern `<token>content<token>`
//! and provide visual emphasis and semantic markup for text content. They use the text
//! transform layer architecture for uniform processing across all text contexts.
//!
//! ## Element Types
//!
//! ### 1. Strong (Bold)
//! - **Syntax**: `*content*`
//! - **Token**: Single asterisk (`*`)
//! - **Purpose**: Strong importance, key concepts, warnings
//! - **Nesting**: Can contain other inline types (except strong)
//!
//! ### 2. Emphasis (Italic)
//! - **Syntax**: `_content_`
//! - **Token**: Single underscore (`_`)
//! - **Purpose**: Emphasis, foreign words, titles, definitions
//! - **Nesting**: Can contain other inline types (except emphasis)
//!
//! ### 3. Code
//! - **Syntax**: `` `content` ``
//! - **Token**: Single backtick (`` ` ``)
//! - **Purpose**: Code, commands, filenames, technical terms
//! - **Nesting**: No nesting allowed (literal content only)
//!
//! ### 4. Math
//! - **Syntax**: `#content#`
//! - **Token**: Single hash (`#`)
//! - **Purpose**: Mathematical expressions and scientific notation
//! - **Nesting**: No nesting allowed (literal content only)
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/formatting/formatting.txxt`]
//! - **General Spec**: [`docs/specs/elements/formatting/inlines-general.txxt`]
//! - **AST Nodes**: [`src/ast/elements/formatting/`]
//! - **Tokenizer**: [`src/lexer/elements/formatting/`]

pub mod code;
pub mod emphasis;
pub mod math;
pub mod strong;

// Re-export formatting parsing functions
pub use code::*;
pub use emphasis::*;
pub use math::*;
pub use strong::*;

use crate::ast::elements::formatting::inlines::{Inline, TextTransform};
use crate::cst::ScannerToken;
use crate::semantic::elements::inlines::InlineParseError;

/// Parse all formatting elements from a sequence of tokens
///
/// This is the main entry point for formatting parsing. It processes a sequence
/// of tokens and returns formatting elements using the text transform layer.
///
/// # Arguments
/// * `tokens` - Sequence of tokens to parse for formatting
///
/// # Returns
/// * `Result<Vec<TextTransform>, InlineParseError>`
///
/// # Processing Order
/// 1. **Code spans** (highest priority - prevents conflicts)
/// 2. **Math expressions** (no further parsing)
/// 3. **Strong elements** (asterisk tokens)
/// 4. **Emphasis elements** (underscore tokens)
/// 5. **Plain text** (default)
pub fn parse_formatting_elements(
    tokens: &[ScannerToken],
) -> Result<Vec<TextTransform>, InlineParseError> {
    let mut transforms = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        if token.is_bold_delimiter() {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_bold_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let nested_transforms = parse_formatting_elements(content_tokens)?;
                transforms.push(TextTransform::Strong(nested_transforms));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else if token.is_italic_delimiter() {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_italic_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let nested_transforms = parse_formatting_elements(content_tokens)?;
                transforms.push(TextTransform::Emphasis(nested_transforms));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else if token.is_code_delimiter() {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_code_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let text = content_tokens.iter().map(|t| t.content()).collect::<String>();
                transforms.push(TextTransform::Code(
                    crate::ast::elements::formatting::inlines::Text::simple(&text),
                ));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else if token.is_math_delimiter() {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_math_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let text = content_tokens.iter().map(|t| t.content()).collect::<String>();
                transforms.push(TextTransform::Math(
                    crate::ast::elements::formatting::inlines::Text::simple(&text),
                ));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else {
            transforms.push(token_to_identity(token));
            i += 1;
        }
    }

    Ok(transforms)
}

fn find_closing_token<P>(tokens: &[ScannerToken], start: usize, predicate: P) -> Option<usize>
where
    P: Fn(&ScannerToken) -> bool,
{
    tokens[start..].iter().position(|token| predicate(token)).map(|pos| start + pos)
}

fn token_to_identity(token: &ScannerToken) -> TextTransform {
    TextTransform::Identity(crate::ast::elements::formatting::inlines::Text::simple(
        token.content(),
    ))
}

/// Parse formatting inline elements and return as Inline variants
///
/// This function processes formatting elements and wraps them in the Inline enum
/// for integration with the broader inline parsing system.
///
/// # Arguments
/// * `tokens` - Sequence of tokens to parse for formatting
///
/// # Returns
/// * `Result<Vec<Inline>, InlineParseError>`
pub fn parse_formatting_inlines(tokens: &[ScannerToken]) -> Result<Vec<Inline>, InlineParseError> {
    let transforms = parse_formatting_elements(tokens)?;

    let inlines = transforms.into_iter().map(Inline::TextLine).collect();

    Ok(inlines)
}
