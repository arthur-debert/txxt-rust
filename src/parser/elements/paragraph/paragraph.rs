//! # Paragraph Parser Module
//!
//! This module contains the logic for parsing paragraphs - the fundamental text blocks
//! that contain inline content and form the basic unit of readable text in txxt documents.
//!
//! ## Overview
//!
//! Paragraphs are the foundational building blocks for textual content in txxt. They contain
//! inline text with formatting, create readable text flow, and serve as the default element
//! type when no other block structure is detected. Paragraphs provide the semantic foundation
//! for document content while supporting rich inline formatting including emphasis, code,
//! references, and mathematical expressions.
//!
//! ## Grammar
//!
//! From [`docs/specs/core/syntax.txxt`]:
//!
//! ```text
//! <paragraph> = <text-line>+ <blank-line>?
//! <text-line> = <span-element> (<whitespace> <span-element>)* <line-break>
//! ```
//!
//! A paragraph consists of one or more consecutive text lines, optionally terminated by a
//! blank line. Lines at the same indentation level continue the paragraph.
//!
//! ## AST Structure
//!
//! Post-parsing semantic representation:
//!
//! ```text
//! Paragraph AST:
//!     ├── ParagraphBlock
//!     │   ├── content: Vec<TextTransform>
//!         Annotations attached to this element, post parsing at assembly (during time aanotations are regular items in container)
//!     │   ├── annotations: Vec<Annotation>
//!     │   ├── parameters: Parameters
//!     │   └── tokens: TokenSequence
//! ```
//!
//! Key structural properties:
//! - Content stored as sequence of inline elements (`TextTransform`)
//! - Text runs merged for efficiency
//! - Formatting preserved through inline element types
//! - Source tokens maintained for reconstruction
//!
//! ## AST Node Declaration
//!
//! ```rust
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//! pub struct ParagraphBlock {
//!     /// Paragraph content with inline formatting
//!     pub content: Vec<TextTransform>,
//!         Annotations attached to this paragraph
//!     pub annotations: Vec<Annotation>,
//!     /// Parameters for this paragraph
//!     pub parameters: Parameters,
//!     /// Raw tokens for precise source reconstruction
//!     pub tokens: TokenSequence,
//! }
//! ```
//!
//! ## Corpora Examples
//!
//! From [`docs/dev/parser-core/per-element-corpora.txxt`] (simple cases only):
//!
//! ### Simple Cases (Basic Forms Only)
//! - `txxt.core.spec.paragraph.valid.simple` - Basic paragraph with plain text
//! - `txxt.core.spec.paragraph.valid.multiline` - Paragraph spanning multiple lines
//! - `txxt.core.spec.paragraph.valid.multiple-with-blanks` - Multiple paragraphs with blank line separation
//! - `txxt.core.spec.paragraph.valid.consistent-indent` - Paragraph with consistent indentation
//ddd
//! ## Ensemble Examples
//!
//! From [`docs/specs/elements/paragraph/examples/`]:
//!
//! - `01-simple-nosession-single-paragraph.txxt` - Single paragraph only
//! - `02-simple-nosession-multiple-paragraph.txxt` - Multiple paragraphs only
//! - `03-simple-flat-single-paragraph.txxt` - Single session with paragraph
//! - `04-simple-flat-multiple-paragraph.txxt` - Multiple sessions with paragraphs
//! - `05-simple-nested-multiple-paragraph.txxt` - Nested sessions with paragraphs
//!
//! ## AST Assertion Example
//!
//! ```rust
//! use tests::assertions::{assert_paragraph, ParagraphExpected};
//!
//! // Minimal validation (one property)
//! assert_paragraph(&element, ParagraphExpected {
//!     text_contains: Some("expected"),
//!     ..Default::default()
//! });
//!
//! // Comprehensive validation (many properties)
//! assert_paragraph(&element, ParagraphExpected {
//!     text: Some("This is a complete paragraph."),
//!     has_formatting: Some(false),
//!     annotation_count: Some(0),
//!     ..Default::default()
//! });
//! ```
//!
//! ## Processing Rules
//!
//! Paragraphs follow the standard processing pattern with these specific requirements:
//!
//! ### Recognition Criteria
//! - Default recognition pattern (lowest priority)
//! - Line does not match any other block element pattern
//! - Line contains text content (not whitespace-only)
//! - Line not at increased indentation from current level
//! - Serves as default element type when no other pattern matches
//!
//! ### Line Grouping
//! 1. Collect consecutive lines at same indentation level
//! 2. Stop at blank line or indentation change
//! 3. Stop at line matching another block element pattern
//! 4. Parse collected lines as single paragraph content
//!
//! ### Inline Processing
//! 1. Concatenate all paragraph lines with spaces
//! 2. Apply standard whitespace normalization
//! 3. Parse through standard inline parser for formatting
//! 4. Merge adjacent text runs using standard efficiency rules
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/paragraph/paragraph.txxt`]
//! - **AST Node**: [`src/ast/elements/paragraph/paragraph.rs`]
//! - **Tokenizer**: [`src/lexer/elements/paragraph.rs`]
//! - **Test Assertions**: [`tests/assertions/elements/paragraph/`]
//! - **Corpora**: [`docs/dev/parser-core/per-element-corpora.txxt`]
//! - **Ensemble Examples**: [`docs/specs/elements/paragraph/examples/`]
//!
//! ## Testing:
//!
//!  1. Test manually the AST generated by the parser for the corpora examples.
//!  2. Use the AST assertions to test the parser for the corpora examples:
//!     2.1 One test per corpora sample.
//!     2.2 Keep in mind that the AST assertions were done pre parsing and are not
//!     being used yet, that is approach it as work in progress and be mindful that
//!     the AST assertions can have bugs or gaps, in which case verify through
//!     the ast testing and then fix / improve the AST assertions.
//!  3. Use the ensemble examples to test the parser for the corpora examples.
//!  3. Use the corpora stack to test exception / errors.
//!
//! ## Paragraph specificity:
//!
//!  Paragraphs are the catchall element, that is parsing a pargraph should never fail, as
//!  long as we have a line with characters.
//!

use crate::ast::{
    elements::{
        components::parameters::Parameters,
        inlines::{TextSpan, TextTransform},
        paragraph::ParagraphBlock,
    },
    tokens::{Token, TokenSequence},
};
use crate::lexer::elements::paragraph::paragraph_tokenizer::{
    collect_paragraph_lines, detect_paragraph, ParagraphParseResult,
};
use crate::parser::pipeline::parse_blocks::BlockParseError;

/// Parse tokens into a ParagraphBlock AST node
///
/// This function takes a sequence of tokens and converts them into a
/// ParagraphBlock AST node. It handles the recognition, line grouping,
/// and inline processing as described in the processing rules.
pub fn parse_paragraph(tokens: &[Token]) -> Result<ParagraphBlock, BlockParseError> {
    // Step 1: Detect if this is a paragraph
    match detect_paragraph(tokens) {
        ParagraphParseResult::ValidParagraph(paragraph) => {
            // Step 2: Process inline content
            let content = parse_inline_content(&paragraph.content)?;

            // Step 3: Create token sequence for reconstruction
            let mut token_sequence = TokenSequence::new();
            token_sequence.tokens = tokens.to_vec();

            // Step 4: Create ParagraphBlock AST node
            Ok(ParagraphBlock::new(
                content,
                Vec::new(),        // Annotations will be attached during assembly
                Parameters::new(), // No parameters for simple paragraphs
                token_sequence,
            ))
        }
        ParagraphParseResult::NotParagraph => Err(BlockParseError::InvalidStructure(
            "Tokens do not represent a paragraph".to_string(),
        )),
        ParagraphParseResult::Invalid(error) => Err(BlockParseError::InvalidStructure(error)),
    }
}

/// Parse multiple lines into a ParagraphBlock AST node
///
/// This function handles multi-line paragraphs by collecting consecutive
/// lines at the same indentation level and processing them as a single
/// paragraph unit.
pub fn parse_multiline_paragraph(
    line_tokens: &[Vec<Token>],
) -> Result<ParagraphBlock, BlockParseError> {
    // Step 1: Collect paragraph lines
    let paragraph = collect_paragraph_lines(line_tokens).ok_or_else(|| {
        BlockParseError::InvalidStructure("No valid paragraph content found".to_string())
    })?;

    // Step 2: Process inline content
    let content = parse_inline_content(&paragraph.content)?;

    // Step 3: Create token sequence from all line tokens
    let all_tokens: Vec<Token> = line_tokens.iter().flatten().cloned().collect();
    let mut token_sequence = TokenSequence::new();
    token_sequence.tokens = all_tokens;

    // Step 4: Create ParagraphBlock AST node
    Ok(ParagraphBlock::new(
        content,
        Vec::new(),        // Annotations will be attached during assembly
        Parameters::new(), // No parameters for simple paragraphs
        token_sequence,
    ))
}

/// Parse inline content from paragraph text
///
/// For Phase 1 Simple Elements, we only handle plain text without
/// inline formatting. This will be expanded in Phase 2.
fn parse_inline_content(
    text: &str,
) -> Result<Vec<crate::ast::elements::inlines::TextTransform>, BlockParseError> {
    // For Phase 1 Simple Elements, we only create plain text transforms
    // Inline formatting will be handled in Phase 2
    if text.trim().is_empty() {
        return Ok(vec![]);
    }

    // Create a simple text transform for the entire content
    let mut text_span = TextSpan {
        tokens: TokenSequence::new(),
        annotations: Vec::new(),
        parameters: Parameters::new(),
    };

    // Set the text content in the token sequence
    text_span.tokens.tokens = vec![Token::Text {
        content: text.to_string(),
        span: crate::ast::tokens::SourceSpan {
            start: crate::ast::tokens::Position { row: 0, column: 0 },
            end: crate::ast::tokens::Position {
                row: 0,
                column: text.len(),
            },
        },
    }];

    let text_transform = TextTransform::Identity(text_span);

    Ok(vec![text_transform])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::elements::inlines::TextTransform;
    use crate::ast::tokens::{Position, SourceSpan};

    fn create_test_span() -> SourceSpan {
        SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 10 },
        }
    }

    fn create_text_token(content: &str) -> Token {
        Token::Text {
            content: content.to_string(),
            span: create_test_span(),
        }
    }

    #[test]
    fn test_parse_simple_paragraph() {
        let tokens = vec![create_text_token("This is a simple paragraph.")];

        let result = parse_paragraph(&tokens);
        assert!(result.is_ok());

        let paragraph = result.unwrap();
        assert_eq!(paragraph.content.len(), 1);
        assert_eq!(paragraph.annotations.len(), 0);
        assert_eq!(paragraph.parameters.map.len(), 0);

        // Check text content
        if let TextTransform::Identity(text_span) = &paragraph.content[0] {
            assert_eq!(text_span.tokens.text(), "This is a simple paragraph.");
        } else {
            panic!("Expected identity text transform");
        }
    }

    #[test]
    fn test_parse_multiline_paragraph() {
        let line1 = vec![create_text_token("This paragraph begins on one line")];
        let line2 = vec![create_text_token("and continues on the next line.")];
        let line_tokens = vec![line1, line2];

        let result = parse_multiline_paragraph(&line_tokens);
        assert!(result.is_ok());

        let paragraph = result.unwrap();
        assert_eq!(paragraph.content.len(), 1);

        // Check combined text content
        if let TextTransform::Identity(text_span) = &paragraph.content[0] {
            assert_eq!(
                text_span.tokens.text(),
                "This paragraph begins on one line and continues on the next line."
            );
        } else {
            panic!("Expected identity text transform");
        }
    }

    #[test]
    fn test_parse_empty_paragraph() {
        let tokens = vec![];

        let result = parse_paragraph(&tokens);
        assert!(result.is_err());

        match result.unwrap_err() {
            BlockParseError::InvalidStructure(msg) => {
                assert!(msg.contains("do not represent a paragraph"));
            }
            _ => panic!("Expected InvalidStructure error"),
        }
    }

    #[test]
    fn test_parse_inline_content() {
        let result = parse_inline_content("Hello world");
        assert!(result.is_ok());

        let content = result.unwrap();
        assert_eq!(content.len(), 1);

        if let TextTransform::Identity(text_span) = &content[0] {
            assert_eq!(text_span.tokens.text(), "Hello world");
        } else {
            panic!("Expected identity text transform");
        }
    }

    #[test]
    fn test_parse_inline_content_empty() {
        let result = parse_inline_content("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
