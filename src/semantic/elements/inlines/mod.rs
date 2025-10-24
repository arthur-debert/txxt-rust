//! # Inline Elements Parser Module
//!
//! This module contains the logic for parsing inline elements - the rich text formatting
//! and reference elements that appear within text blocks like paragraphs.
//!
//! ## Overview
//!
//! Inline elements provide rich text formatting and semantic markup within text blocks.
//! They follow the general token-based pattern `<token>content<token>` and are processed
//! during Phase 2b of the parsing pipeline. The text transform layer provides uniform
//! handling for all text content across different contexts.
//!
//! ## Architecture
//!
//! Inline parsing uses a text transform layer architecture where every piece of text
//! goes through a transform layer for uniform processing:
//!
//! ```text
//! Text Processing Flow:
//! Raw Text → Token Detection → Transform Parsing → AST Construction
//! ```
//!
//! This enables:
//! - Consistent handling of text content across all inline types
//! - Extensibility for new transform types without changing core structure
//! - Composability for nested formatting (e.g., **_bold italic_**)
//! - Language server precision through token sequences
//!
//! ## Element Categories
//!
//! ### 1. Formatting Elements
//! - **Strong (Bold)**: `*content*` - High-priority information
//! - **Emphasis (Italic)**: `_content_` - Stressed or distinguished content
//! - **Code**: `` `content` `` - Technical content and literal text
//! - **Math**: `#content#` - Mathematical expressions
//!
//! ### 2. Reference Elements
//! - **Citations**: `[@smith2023]`, `[@doe2024; @jones2025]`
//! - **Footnotes**: `[1]`, `[^note-label]`
//! - **Page References**: `[page:123]`, `[pages:123-125]`
//! - **Session References**: `[#3]`, `[#2.1]`, `[local-section]`
//! - **General References**: `[filename.txxt]`, `[https://example.com]`
//!
//! ## Grammar
//!
//! From [`docs/specs/elements/formatting/inlines-general.txxt`]:
//!
//! ```text
//! <inline-element> = <token>content<token>
//!
//! Formatting Spans:
//! <bold-span> = <asterisk> <text-content> <asterisk>
//! <italic-span> = <underscore> <text-content> <underscore>
//! <code-span> = <backtick> <text-content> <backtick>
//! <math-span> = <hash> <text-content> <hash>
//!
//! Reference Spans:
//! <reference-span> = <left-bracket> <reference-content> <right-bracket>
//! <citation-span> = <left-bracket> <at-sign> <citation-keys> <citation-locator>? <right-bracket>
//! <page-ref> = <left-bracket> <page-locator> <right-bracket>
//! <session-ref> = <left-bracket> <hash> <session-number> <right-bracket>
//! <footnote-ref> = <footnote-naked> | <footnote-labeled>
//! ```
//!
//! ## AST Structure
//!
//! Post-parsing semantic representation using the text transform layer:
//!
//! ```text
//! Inline AST:
//!     ├── Inline
//!     │   ├── TextLine(TextTransform)
//!     │   │   ├── Identity(Text)           # Plain text
//!     │   │   ├── Emphasis(Vec<TextTransform>)  # Italic formatting
//!     │   │   ├── Strong(Vec<TextTransform>)    # Bold formatting
//!     │   │   ├── Code(Text)               # Code formatting
//!     │   │   ├── Math(Text)               # Math expressions
//!     │   │   └── Composed(Vec<TextTransform>) # Nested formatting
//!     │   ├── Reference
//!     │   │   ├── target: ReferenceTarget
//!     │   │   ├── content: Option<Vec<Inline>>
//!     │   │   └── tokens: ScannerTokenSequence
//!     │   └── Link
//!     │       ├── target: String
//!     │       ├── content: Vec<Inline>
//!     │       └── tokens: ScannerTokenSequence
//! ```
//!
//! ## Parsing Priority
//!
//! Element parsing order to resolve conflicts:
//! 1. **Code spans** (highest priority - no further parsing)
//! 2. **Math expressions** (no further parsing)
//! 3. **References** (validate target format)
//! 4. **Formatting elements** (allow nesting)
//! 5. **Plain text** (default)
//!
//! ## Processing Rules
//!
//! ### Recognition Criteria
//! - Token-based detection: `<token>content<token>`
//! - No spaces between token and content boundaries
//! - Content cannot be empty
//! - Content cannot span line breaks (single-line only)
//! - Nested inline elements must be different types
//!
//! ### Nesting Rules
//! ```text
//! Valid: *strong with `code` inside*
//! Valid: _emphasis with #math# inside_
//! Invalid: *strong with *nested strong* inside*
//! Invalid: `code with `nested code` inside`
//! ```
//!
//! ### Error Recovery
//! - Unbalanced tokens → Treat as literal text
//! - Empty content → Parse error, skip element
//! - Invalid nesting → Break at conflict point
//! - Unknown token pattern → Preserve as text
//!
//! ## Integration with Block Elements
//!
//! Inline processing contexts:
//! - **Paragraphs**: Primary container for inline content
//! - **List items**: Rich inline formatting support
//! - **Definition terms**: Inline formatting allowed
//! - **Annotation content**: Full inline support
//!
//! Processing flow:
//! 1. Parse block structure first
//! 2. Process inline content within blocks
//! 3. Maintain block context for reference resolution
//! 4. Integrate with document-wide systems
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/formatting/inlines-general.txxt`]
//! - **Formatting Spec**: [`docs/specs/elements/formatting/formatting.txxt`]
//! - **References Spec**: [`docs/specs/elements/references/references-general.txxt`]
//! - **AST Nodes**: [`src/ast/elements/inlines/`]
//! - **AST Formatting**: [`src/ast/elements/formatting/`]
//! - **AST References**: [`src/ast/elements/references/`]
//! - **Tokenizer**: [`src/lexer/elements/inline/`]
//!
//! ## Testing Strategy
//!
//! 1. **Phase 1**: Formatting inlines (bold, italic, code, math)
//!    - Single inline per paragraph
//!    - Test isolated cases
//! 2. **Phase 2**: Multiple inlines per paragraph
//!    - Test complex combinations
//!    - Test nesting scenarios
//! 3. **Phase 3**: References integration
//!    - Citations, footnotes, page refs, session refs
//!    - Mixed content with formatting
//!
//! ## Implementation Notes
//!
//! - Inline parsing happens in Phase 2b of the parsing pipeline
//! - Focus on paragraph content processing initially
//! - Use existing tokenizer inline detection
//! - Maintain token-level precision for language server support
//! - Support progressive complexity (single → multiple → nested)

pub mod engine;
pub mod references;

// Re-export inline parsing functions
pub use references::*;

/// Parse inline elements from a sequence of tokens
///
/// This is the main entry point for inline parsing. It processes a sequence
/// of tokens and returns a vector of inline elements using the generic
/// inline engine.
///
/// # Arguments
/// * `tokens` - Sequence of tokens to parse as inline elements
///
/// # Returns
/// * `Result<Vec<crate::ast::elements::formatting::inlines::Inline>, InlineParseError>`
///
/// See tests/parser/elements/inlines/test_formatting.rs for examples
pub fn parse_inlines(
    tokens: &[crate::cst::ScannerToken],
) -> Result<Vec<crate::ast::elements::formatting::inlines::Inline>, InlineParseError> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    // Use the generic inline engine
    let engine = engine::create_standard_engine()
        .map_err(|e| InlineParseError::InvalidStructure(e.to_string()))?;

    Ok(engine.parse(tokens))
}

/// Parse formatting inline elements (bold, italic, code, math)
///
/// This function handles the parsing of formatting elements that use the
/// `<token>content<token>` pattern for visual emphasis and semantic markup.
///
/// # Arguments
/// * `tokens` - Sequence of tokens to parse for formatting
///
/// # Returns
/// * `Result<Vec<crate::ast::elements::formatting::inlines::TextTransform>, InlineParseError>`
///
/// # Formatting Types
/// * **Strong (Bold)**: `*content*` - Single asterisk tokens
/// * **Emphasis (Italic)**: `_content_` - Single underscore tokens
/// * **Code**: `` `content` `` - Single backtick tokens
/// * **Math**: `#content#` - Single hash tokens
///
/// # Note
/// This function filters the engine output to only return TextTransform variants.
/// References and other non-formatting inlines are excluded.
pub fn parse_formatting(
    tokens: &[crate::cst::ScannerToken],
) -> Result<Vec<crate::ast::elements::formatting::inlines::TextTransform>, InlineParseError> {
    let inlines = parse_inlines(tokens)?;

    // Extract only TextTransform variants
    let transforms = inlines
        .into_iter()
        .filter_map(|inline| match inline {
            crate::ast::elements::formatting::inlines::Inline::TextLine(transform) => {
                Some(transform)
            }
            _ => None, // Skip references and other non-formatting inlines
        })
        .collect();

    Ok(transforms)
}

/// Parse reference inline elements (citations, footnotes, page refs, session refs)
///
/// This function handles the parsing of reference elements that use square
/// bracket tokens `[content]` for cross-references and citations.
///
/// # Arguments
/// * `tokens` - Sequence of tokens to parse for references
///
/// # Returns
/// * `Result<Vec<crate::ast::elements::formatting::inlines::Inline>, InlineParseError>`
///
/// # Reference Types
/// * **Citations**: `[@smith2023]`, `[@doe2024; @jones2025]`
/// * **Footnotes**: `[1]`, `[^note-label]`
/// * **Page References**: `[page:123]`, `[pages:123-125]`
/// * **Session References**: `[#3]`, `[#2.1]`, `[local-section]`
/// * **General References**: `[filename.txxt]`, `[https://example.com]`
pub fn parse_references(
    tokens: &[crate::cst::ScannerToken],
) -> Result<Vec<crate::ast::elements::formatting::inlines::Inline>, InlineParseError> {
    // TODO: Implement reference parsing logic
    // For now, return a placeholder

    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    // Convert all tokens to plain text and preserve token sequence
    let text_content = tokens
        .iter()
        .filter_map(|token| match token {
            crate::cst::ScannerToken::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    if text_content.is_empty() {
        return Ok(Vec::new());
    }

    // Create token sequence from source tokens
    let token_sequence = crate::cst::ScannerTokenSequence {
        tokens: tokens.to_vec(),
    };

    // Create a text inline preserving source tokens
    let text_inline = crate::ast::elements::formatting::inlines::Inline::TextLine(
        crate::ast::elements::formatting::inlines::TextTransform::Identity(
            crate::ast::elements::formatting::inlines::Text::simple_with_tokens(
                &text_content,
                token_sequence,
            ),
        ),
    );

    Ok(vec![text_inline])
}

/// Errors that can occur during inline parsing
#[derive(Debug)]
pub enum InlineParseError {
    /// Invalid inline structure detected
    InvalidStructure(String),
    /// Unbalanced tokens (missing closing token)
    UnbalancedTokens(String),
    /// Empty content between tokens
    EmptyContent(String),
    /// Invalid nesting detected
    InvalidNesting(String),
    /// Unknown token pattern
    UnknownTokenPattern(String),
    /// Reference target parsing error
    ReferenceTargetError(String),
    /// Citation parsing error
    CitationError(String),
    /// Math expression parsing error
    MathError(String),
}

impl std::fmt::Display for InlineParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineParseError::InvalidStructure(msg) => {
                write!(f, "Invalid inline structure: {}", msg)
            }
            InlineParseError::UnbalancedTokens(msg) => write!(f, "Unbalanced tokens: {}", msg),
            InlineParseError::EmptyContent(msg) => write!(f, "Empty content: {}", msg),
            InlineParseError::InvalidNesting(msg) => write!(f, "Invalid nesting: {}", msg),
            InlineParseError::UnknownTokenPattern(msg) => {
                write!(f, "Unknown token pattern: {}", msg)
            }
            InlineParseError::ReferenceTargetError(msg) => {
                write!(f, "Reference target error: {}", msg)
            }
            InlineParseError::CitationError(msg) => write!(f, "Citation error: {}", msg),
            InlineParseError::MathError(msg) => write!(f, "Math expression error: {}", msg),
        }
    }
}

impl std::error::Error for InlineParseError {}
