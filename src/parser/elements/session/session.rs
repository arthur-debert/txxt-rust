//! # Session Parser Module
//!
//! This module contains the logic for parsing sessions - the hierarchical organizational
//! units that structure txxt documents into navigable sections.
//!
//! ## Overview
//!
//! Sessions are the primary organizational element in txxt documents, providing hierarchical
//! structure similar to chapters, sections, and subsections. They enable document navigation,
//! content organization, and automated table of contents generation. Sessions can be numbered
//! or unnumbered and support arbitrary nesting depth with flexible numbering schemes.
//!
//! ## Grammar
//!
//! From [`docs/specs/core/syntax.txxt`]:
//!
//! ```text
//! <session> = <session-title> <blank-line> <session-container>
//! <session-title> = <session-numbering>? <text-line>
//! <session-numbering> = <sequence-marker> <whitespace>
//! ```
//!
//! Sessions require blank line separation and indented content to distinguish them from
//! paragraphs. Session numbering uses the same sequence marker patterns as lists.
//!
//! ## AST Structure
//!
//! Post-parsing semantic representation:
//!
//! ```text
//! Session AST:
//!     ├── SessionBlock
//!     │   ├── title: SessionTitle
//!     │   │   ├── content: Vec<Inline>
//!     │   │   ├── numbering: Option<SessionNumbering>
//!     │   │   └── tokens: TokenSequence
//!     │   ├── content: SessionContainer
//!     │   │   └── content: Vec<Block>
//!     │   ├── annotations: Vec<Annotation>
//!     │   └── tokens: TokenSequence
//! ```
//!
//! Key structural properties:
//! - Title stored as inline content (supports formatting)
//! - Content stored in Session Container (can hold any blocks including sessions)
//! - Numbering information preserved exactly for source reconstruction
//! - Recursive structure enables arbitrary nesting depth
//!
//! ## AST Node Declaration
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//! pub struct SessionBlock {
//!     /// Session title with optional numbering
//!     pub title: SessionTitle,
//!     /// Session content container
//!     pub content: SessionContainer,
//!         Annotations attached to this element, post parsing at assembly (during time aanotations are regular items in container)
//!     pub annotations: Vec<Annotation>,
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
//! - `txxt.core.spec.session.valid.unnumbered-basic` - Unnumbered session with content
//! - `txxt.core.spec.session.valid.numbered-basic` - Numbered session with content
//! - `txxt.core.spec.session.valid.flat-one-child` - Session with single child element
//! - `txxt.core.spec.session.valid.flat-two-children` - Session with two child elements
//!
//! ## Ensemble Examples
//!
//! From [`docs/specs/elements/session/examples/`]:
//!
//! - `01-simple-nosession-single-session.txxt` - Single session only
//! - `02-simple-nosession-multiple-session.txxt` - Multiple sessions only
//! - `03-simple-flat-single-session.txxt` - Single session with content
//! - `04-simple-flat-multiple-session.txxt` - Multiple sessions with content
//! - `05-simple-nested-multiple-session.txxt` - Nested sessions with content
//!
//! ## AST Assertion Example
//!
//! ```rust,ignore
//! use tests::assertions::{assert_session, SessionExpected};
//!
//! // Minimal validation (one property)
//! assert_session(&element, SessionExpected {
//!     title_contains: Some("Introduction"),
//!     ..Default::default()
//! });
//!
//! // Comprehensive validation (many properties)
//! assert_session(&element, SessionExpected {
//!     title: Some("1. Getting Started"),
//!     has_numbering: Some(true),
//!     child_count: Some(2),
//!     ..Default::default()
//! });
//! ```
//!
//! ## Processing Rules
//!
//! Sessions follow the standard processing pattern with these specific requirements:
//!
//! ### Recognition Criteria
//! - Preceded by blank line (or start of document)
//! - Followed by indented content (+1 indentation level)
//! - Without indented content → Parsed as paragraph
//! - Title can contain numbering markers (1., a., i., etc.)
//!
//! ### Session Detection
//! 1. Check for preceding blank line (or document start)
//! 2. Parse potential title line
//! 3. Check for indented content following title
//! 4. If indented content exists → Session
//! 5. If no indented content → Apply graceful degradation (paragraph)
//!
//! ### Content Processing
//! 1. Create Session Container for indented content
//! 2. Apply standard recursive parsing (including nested sessions)
//! 3. Apply blank line separation rules for nested sessions
//! 4. Validate content types (all blocks allowed - Session Container rules)
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/session/session.txxt`]
//! - **AST Node**: [`src/ast/elements/session/session.rs`]
//! - **Tokenizer**: [`src/lexer/elements/session.rs`]
//! - **Test Assertions**: [`tests/assertions/elements/session/`]
//! - **Corpora**: [`docs/dev/parser-core/per-element-corpora.txxt`]
//! - **Ensemble Examples**: [`docs/specs/elements/session/examples/`]
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
//! ## Session specificity:
//!
//!  Sessions require indented content to be recognized as sessions. Without indented
//!  content, they fall back to paragraph parsing. This disambiguation rule ensures
//!  clear structural intent and prevents accidental session creation.
//!

use crate::ast::{
    elements::{
        session::{block::SessionBlock, session_container::SessionContainer},
        tokens::{Token, TokenSequence},
    },
    ElementNode,
};
use crate::parser::pipeline::parse_blocks::BlockParseError;

/// Parse a session from a sequence of tokens
///
/// A session consists of:
/// 1. A title line (with optional numbering)
/// 2. A blank line
/// 3. Indented content (at least one level deeper)
///
/// Returns an error if the structure is not a valid session.
pub fn parse_session(tokens: &[Token]) -> Result<SessionBlock, BlockParseError> {
    if tokens.is_empty() {
        return Err(BlockParseError::InvalidStructure(
            "Empty token sequence".to_string(),
        ));
    }

    // Find the title line (first non-blank token)
    let title_start = tokens
        .iter()
        .position(|t| !matches!(t, Token::BlankLine { .. }));
    let title_start = match title_start {
        Some(pos) => pos,
        None => {
            return Err(BlockParseError::InvalidStructure(
                "No title found".to_string(),
            ))
        }
    };

    // Find the end of the title line (next blank line or end of tokens)
    let title_end = tokens[title_start..]
        .iter()
        .position(|t| matches!(t, Token::BlankLine { .. }))
        .map(|pos| title_start + pos)
        .unwrap_or(tokens.len());

    if title_end == title_start {
        return Err(BlockParseError::InvalidStructure("Empty title".to_string()));
    }

    // Extract title tokens
    let title_tokens = &tokens[title_start..title_end];

    // Parse the title
    let title = parse_session_title(title_tokens)?;

    // Find content after the blank line
    let content_start = title_end + 1; // Skip the blank line
    if content_start >= tokens.len() {
        return Err(BlockParseError::InvalidStructure(
            "No content found after title".to_string(),
        ));
    }

    // Extract content tokens (everything after the blank line)
    let content_tokens = &tokens[content_start..];

    // Parse the content as a SessionContainer
    let content = parse_session_content(content_tokens)?;

    // Create the session block
    let session = SessionBlock {
        title,
        content,
        annotations: Vec::new(), // TODO: Parse annotations when implemented
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: TokenSequence::new(),
    };

    Ok(session)
}

/// Parse a session title from tokens
fn parse_session_title(
    tokens: &[Token],
) -> Result<crate::ast::elements::session::block::SessionTitle, BlockParseError> {
    if tokens.is_empty() {
        return Err(BlockParseError::InvalidStructure(
            "Empty title tokens".to_string(),
        ));
    }

    // For now, create a simple title without numbering detection
    // TODO: Implement proper numbering detection and parsing
    let title = crate::ast::elements::session::block::SessionTitle {
        content: Vec::new(), // TODO: Parse inline content
        numbering: None,     // TODO: Detect and parse numbering
        tokens: TokenSequence::new(),
    };

    Ok(title)
}

/// Parse session content as a SessionContainer
fn parse_session_content(tokens: &[Token]) -> Result<SessionContainer, BlockParseError> {
    // Create a token tree from the content tokens
    // For now, we'll create a simple flat structure
    let token_tree = crate::lexer::pipeline::TokenTree {
        tokens: tokens.to_vec(),
        children: Vec::new(), // TODO: Handle nested indentation properly
    };

    // Use the block parser to parse the content
    let block_parser = crate::parser::pipeline::parse_blocks::BlockParser::new();
    let elements = block_parser.parse_blocks(token_tree)?;

    // Convert ElementNode to SessionContainerElement
    let content: Vec<crate::ast::elements::session::session_container::SessionContainerElement> = elements
        .into_iter()
        .map(|element| match element {
            ElementNode::ParagraphBlock(paragraph) => {
                Ok(crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(paragraph))
            }
            ElementNode::SessionBlock(session) => {
                Ok(crate::ast::elements::session::session_container::SessionContainerElement::Session(session))
            }
            // TODO: Handle other element types as they're implemented
            _ => {
                // For now, skip unsupported elements
                Err(BlockParseError::InvalidStructure(format!("Unsupported element in session content: {:?}", element)))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Create the session container
    let container = SessionContainer {
        content,
        annotations: Vec::new(), // TODO: Parse annotations when implemented
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: TokenSequence::new(),
    };

    Ok(container)
}
