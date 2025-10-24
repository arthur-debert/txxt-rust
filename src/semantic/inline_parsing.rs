//! # Phase 2b: Inline Parsing
//!
//! ============================================================================
//! OVERVIEW
//! ============================================================================
//!
//! Handles inline elements within block content. This is the second step
//! of Phase 2 parsing, where we take AST block elements and process
//! any inline formatting, references, and other inline elements within them.
//!
//! Inlines are isolated and require no context - they operate on simple text
//! spans within blocks and produce specialized text spans. This makes them
//! much simpler than block elements and enables independent testing and
//! potential parallelization.
//!
//!
//! ============================================================================
//! ARCHITECTURE: THREE-LEVEL DECLARATIVE PIPELINE
//! ============================================================================
//!
//! The inline parsing system uses a declarative three-level pipeline where each
//! level has a distinct responsibility. Components are defined as composable
//! traits rather than monolithic parsing logic.
//!
//!
//! LEVEL 1: DELIMITER MATCHING
//!
//! Purpose: Identify spans enclosed by delimiter pairs
//! Input:   Vec<ScannerToken> (from block elements)
//! Output:  Vec<SpanMatch> (matched delimiter pairs)
//! Module:  semantic::elements::inlines::level1_matchers
//!
//! Key Components:
//! - GenericDelimiterMatcher: Single configurable matcher for all inline types
//! - Factory functions: bold_matcher(), italic_matcher(), code_matcher(),
//!   math_matcher(), reference_matcher()
//!
//! Responsibilities:
//! - Match start and end delimiters (e.g., `*...*`, `[...]`)
//! - Enforce single-line constraint (no newlines in content)
//! - Validate non-empty content
//! - Preserve token sequences for AST construction
//!
//! Examples:
//! - Bold: `*text*` → SpanMatch { start: 0, end: 3, inner: ["text"] }
//! - Reference: `[url]` → SpanMatch { start: 0, end: 3, inner: ["url"] }
//!
//!
//! LEVEL 2: TYPE CLASSIFICATION
//!
//! Purpose: Determine specific inline element type from matched spans
//! Input:   SpanMatch (from Level 1)
//! Output:  TypedSpan (span + InlineType)
//! Module:  semantic::elements::inlines::level2_classifiers
//!
//! Key Components:
//! - FormattingClassifier: Maps delimiter names to formatting types
//!   (bold → InlineType::Bold, italic → InlineType::Italic, etc.)
//!
//! - ReferenceTypeClassifier: Analyzes reference content to determine type
//!   (Uses existing ReferenceClassifier for pattern matching)
//!   - @key → InlineType::Citation
//!   - #3 → InlineType::Section
//!   - [1] or ^label → InlineType::Footnote
//!   - https://... → InlineType::Url
//!   - ./file.txt → InlineType::File
//!   - TK → InlineType::ToComeTK
//!   - other → InlineType::NotSure
//!
//! Responsibilities:
//! - Classify formatting elements (trivial: type = delimiter)
//! - Classify reference elements (complex: analyze content patterns)
//! - Handle ambiguous cases with precedence rules
//!
//!
//! LEVEL 3: DEEP PROCESSING
//!
//! Purpose: Build final AST nodes with complete semantic information
//! Input:   TypedSpan (from Level 2)
//! Output:  Inline (final AST node)
//! Module:  semantic::elements::inlines::level3_processors
//!
//! Key Components (Processors):
//!
//! Formatting Processors:
//! - BoldProcessor: Builds Strong transforms, allows nested different types
//! - ItalicProcessor: Builds Emphasis transforms, prevents same-type nesting
//! - CodeProcessor: Builds Code transforms, no nesting (literal content)
//! - MathProcessor: Builds Math transforms, no nesting (literal content)
//!
//! Reference Processors:
//! - CitationProcessor: Parses citation keys and locators
//!   `@smith2023, p. 123; @jones2025` → Vec<CitationEntry>
//!
//! - FootnoteProcessor: Handles naked numerical and labeled footnotes
//!   `1` → NakedNumerical, `^label` → NamedAnchor
//!
//! - SectionProcessor: Parses section identifiers with hierarchy
//!   `#3` → Numeric([3]), `#2.1.3` → Numeric([2,1,3]), `#-1` → negative index
//!
//! - UrlProcessor: Parses URLs with optional fragments
//!   `https://example.com#section` → Url { url, fragment }
//!
//! - FileProcessor: Parses file paths with optional section anchors
//!   `./file.txt#section` → File { path, section }
//!
//! - TKProcessor: Handles TK (To Come) placeholders
//! - NotSureProcessor: Handles unresolved references
//!
//! Responsibilities:
//! - Parse internal structure (citation keys, section numbers, etc.)
//! - Handle nested formatting recursively
//! - Build complete AST nodes with semantic information
//! - Preserve token sequences for source reconstruction
//!
//!
//! ============================================================================
//! PIPELINE ORCHESTRATION
//! ============================================================================
//!
//! The InlinePipeline struct coordinates all three levels:
//!
//! Module: semantic::elements::inlines::pipeline
//!
//! Key Components:
//! - InlinePipeline: Main orchestrator that runs Level 1 → 2 → 3
//! - create_standard_pipeline(): Factory with all built-in matchers
//! - Priority order: Code > Math > Reference > Bold > Italic
//!   (Code first to prevent conflicts with other delimiters)
//!
//! Processing Flow:
//! ```text
//! ScannerToken[] → [Level 1: Match] → SpanMatch[]
//!                → [Level 2: Classify] → TypedSpan[]
//!                → [Level 3: Process] → Inline[]
//! ```
//!
//! Example: Bold with nested italic
//! ```text
//! Input:  [*, _, "text", _, *]
//!
//! Level 1: Match bold delimiters
//!   → SpanMatch { matcher: "bold", inner: [_, "text", _] }
//!
//! Level 2: Classify as Bold
//!   → TypedSpan { type: Bold, span: ... }
//!
//! Level 3: Process with recursion
//!   → BoldProcessor sees inner italic delimiters
//!   → Recursively processes: Italic("text")
//!   → Output: Strong(Emphasis(Identity("text")))
//! ```
//!
//! Example: Citation with multiple keys
//! ```text
//! Input:  [[, "@smith2023; @jones2025", ]]
//!
//! Level 1: Match reference delimiters
//!   → SpanMatch { matcher: "reference", inner: ["@smith2023; @jones2025"] }
//!
//! Level 2: Classify as Citation (sees @ at start)
//!   → TypedSpan { type: Citation, span: ... }
//!
//! Level 3: Process with citation parser
//!   → CitationProcessor splits by semicolon
//!   → Parses each key: "smith2023", "jones2025"
//!   → Output: Reference(Citation([entry1, entry2]))
//! ```
//!
//!
//! ============================================================================
//! DESIGN PRINCIPLES
//! ============================================================================
//!
//! DECLARATIVE OVER IMPERATIVE
//! - Inline types defined as trait implementations, not hard-coded logic
//! - Adding new inline types: implement 3 traits + factory function
//! - No monolithic switch statements or if-else chains
//!
//! SEPARATION OF CONCERNS
//! - Level 1: Knows about delimiters, not AST
//! - Level 2: Knows about types, not content parsing
//! - Level 3: Knows about semantics, not delimiter matching
//!
//! TESTABILITY
//! - Each level tested independently with unit tests
//! - Mock data at level boundaries (SpanMatch, TypedSpan)
//! - Integration tests verify full pipeline
//!
//! EXTENSIBILITY
//! - Plugin systems can add custom inline types
//! - Reuse GenericDelimiterMatcher for simple cases
//! - Implement custom processors for complex parsing
//!
//! COMPOSABILITY
//! - Nested formatting via recursive processing
//! - Processors can invoke sub-parsers (citations in footnotes, etc.)
//! - Multiple levels of processing (currently 3, extensible if needed)
//!
//!
//! ============================================================================
//! RELATED SPECIFICATIONS
//! ============================================================================
//!
//! - docs/specs/elements/formatting/inlines-general.txxt
//! - docs/specs/elements/formatting/formatting.txxt
//! - docs/specs/elements/references/references-general.txxt
//! - docs/specs/elements/references/citations.txxt
//!
//!
//! ============================================================================
//! SEE ALSO
//! ============================================================================
//!
//! For the complete architecture overview: src/lib.rs
//! For block element parsing: src/semantic/mod.rs
//! For tokenization: src/syntax/mod.rs

use crate::ast::elements::formatting::inlines::{Inline, Text, TextTransform};
use crate::ast::ElementNode;
use crate::semantic::elements::inlines::engine::{create_standard_engine, InlineEngine};

/// Inline parser for processing inline elements within blocks
///
/// This parser takes AST block elements and processes any inline
/// formatting, references, and other inline elements within them.
pub struct InlineParser {
    engine: InlineEngine,
}

impl Default for InlineParser {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineParser {
    /// Create a new inline parser instance
    pub fn new() -> Self {
        // Create engine with all standard inline types registered
        let engine = create_standard_engine()
            .expect("Failed to create standard inline engine - this is a bug");

        Self { engine }
    }

    /// Parse inline elements within block AST nodes
    ///
    /// Takes AST block elements and processes any inline formatting,
    /// references, and other inline elements within their content.
    /// Returns the same AST structure but with inlines processed.
    pub fn parse_inlines(
        &self,
        blocks: Vec<ElementNode>,
    ) -> Result<Vec<ElementNode>, InlineParseError> {
        blocks
            .into_iter()
            .map(|node| self.parse_inlines_in_node(node))
            .collect()
    }

    fn parse_inlines_in_node(&self, node: ElementNode) -> Result<ElementNode, InlineParseError> {
        match node {
            ElementNode::ParagraphBlock(mut block) => {
                // Use the generic inline engine to parse all inline elements
                let inlines = self.engine.parse(&block.tokens.tokens);

                // Convert to TextTransform for backward compatibility
                // TODO: Update ParagraphBlock to support Vec<Inline> directly
                block.content = inlines_to_text_transforms(inlines);
                Ok(ElementNode::ParagraphBlock(block))
            }
            _ => Ok(node),
        }
    }
}

/// Convert Vec<Inline> to Vec<TextTransform> for backward compatibility
///
/// This helper function extracts TextTransform elements from Inline::TextLine variants.
/// Reference elements are currently converted to plain text since the ParagraphBlock
/// structure doesn't yet support mixed Inline content.
///
/// TODO: Update ParagraphBlock.content to Vec<Inline> to properly support references
fn inlines_to_text_transforms(inlines: Vec<Inline>) -> Vec<TextTransform> {
    inlines
        .into_iter()
        .map(|inline| match inline {
            Inline::TextLine(transform) => transform,
            Inline::Reference(reference) => {
                // Convert reference to plain text for now
                // Eventually ParagraphBlock should support Vec<Inline>
                let text = reference.target.display_text();
                TextTransform::Identity(Text::simple_with_tokens(&text, reference.tokens))
            }
            Inline::Link { target, tokens, .. } => {
                // Convert link to plain text for now
                TextTransform::Identity(Text::simple_with_tokens(&target, tokens))
            }
            Inline::Custom { name, tokens, .. } => {
                // Convert custom inline to plain text
                TextTransform::Identity(Text::simple_with_tokens(&name, tokens))
            }
        })
        .collect()
}

/// Errors that can occur during inline parsing
#[derive(Debug)]
pub enum InlineParseError {
    /// Invalid inline structure detected
    InvalidStructure(String),
    /// Unsupported inline type encountered
    UnsupportedInlineType(String),
    /// Parse error at specific position
    ParseError {
        position: crate::cst::Position,
        message: String,
    },
    /// Reference resolution error
    ReferenceError(String),
    /// Generic parse error
    GenericParseError(String),
}

impl From<crate::semantic::elements::inlines::InlineParseError> for InlineParseError {
    fn from(err: crate::semantic::elements::inlines::InlineParseError) -> Self {
        InlineParseError::GenericParseError(err.to_string())
    }
}

impl std::fmt::Display for InlineParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineParseError::InvalidStructure(msg) => {
                write!(f, "Invalid inline structure: {}", msg)
            }
            InlineParseError::UnsupportedInlineType(inline_type) => {
                write!(f, "Unsupported inline type: {}", inline_type)
            }
            InlineParseError::ParseError { position, message } => {
                write!(f, "Parse error at position {:?}: {}", position, message)
            }
            InlineParseError::ReferenceError(reference) => {
                write!(f, "Reference resolution error: {}", reference)
            }
            InlineParseError::GenericParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for InlineParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_parser_creation() {
        let _parser = InlineParser::new();
        // Basic test to ensure parser can be created
        // The test passes if we reach this point without panicking
    }

    #[test]
    fn test_parse_inlines_placeholder() {
        let parser = InlineParser::new();
        let blocks = vec![];

        // This should return the blocks unchanged until Phase 2 is implemented
        let result = parser.parse_inlines(blocks.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), blocks);
    }
}
