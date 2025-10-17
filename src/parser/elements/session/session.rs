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
        inlines::{TextSpan, TextTransform},
        list::{NumberingForm, NumberingStyle},
        session::{
            block::{SessionBlock, SessionTitle},
            session_container::{SessionContainer, SessionContainerElement},
            SessionNumbering,
        },
        tokens::{SequenceMarkerType, Token, TokenSequence},
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

    // Create token sequence for the session
    let mut session_tokens = TokenSequence::new();
    session_tokens.tokens = tokens.to_vec();

    // Create the session block
    let session = SessionBlock {
        title,
        content,
        annotations: Vec::new(), // TODO: Parse annotations when implemented
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: session_tokens,
    };

    Ok(session)
}

/// Parse a session title from tokens
fn parse_session_title(tokens: &[Token]) -> Result<SessionTitle, BlockParseError> {
    if tokens.is_empty() {
        return Err(BlockParseError::InvalidStructure(
            "Empty title tokens".to_string(),
        ));
    }

    let mut numbering = None;
    let content_tokens;

    // Check for sequence marker at the beginning
    if let Some(Token::SequenceMarker {
        marker_type,
        span: _,
        ..
    }) = tokens.first()
    {
        let (style, form, marker) = match marker_type {
            SequenceMarkerType::Plain(s) => (NumberingStyle::Plain, NumberingForm::Short, s),
            SequenceMarkerType::Numerical(_, s) => {
                (NumberingStyle::Numerical, NumberingForm::Short, s)
            }
            SequenceMarkerType::Alphabetical(_, s) => {
                (NumberingStyle::Alphabetical, NumberingForm::Short, s)
            }
            SequenceMarkerType::Roman(_, s) => (NumberingStyle::Roman, NumberingForm::Short, s),
        };

        numbering = Some(SessionNumbering {
            marker: marker.clone(),
            style,
            form,
        });
        content_tokens = &tokens[1..];
    } else {
        content_tokens = tokens;
    }

    // Parse inline content for the title using proper inline parser
    let content = parse_inline_title_content(content_tokens)?;

    // Create token sequence preserving all tokens
    let mut token_sequence = TokenSequence::new();
    token_sequence.tokens = tokens.to_vec();

    Ok(SessionTitle {
        content,
        numbering,
        tokens: token_sequence,
    })
}

/// Parse inline content for a session title from tokens
fn parse_inline_title_content(tokens: &[Token]) -> Result<Vec<TextTransform>, BlockParseError> {
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    // For now, create a simple text span from all tokens
    // TODO: Implement proper inline parsing with formatting support
    let text_content = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            Token::Whitespace { content, .. } => Some(content.as_str()),
            Token::Period { .. } => Some("."),
            Token::Newline { .. } => Some("\n"),
            _ => None, // Skip other tokens for now
        })
        .collect::<Vec<_>>()
        .join("");

    if text_content.trim().is_empty() {
        return Ok(vec![]);
    }

    let mut text_span = TextSpan {
        tokens: TokenSequence::new(),
        annotations: Vec::new(),
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
    };

    // Create a single text token from the content
    text_span.tokens.tokens = vec![Token::Text {
        content: text_content.clone(),
        span: crate::ast::tokens::SourceSpan {
            start: crate::ast::tokens::Position { row: 0, column: 0 },
            end: crate::ast::tokens::Position {
                row: 0,
                column: text_content.len(),
            },
        },
    }];

    Ok(vec![TextTransform::Identity(text_span)])
}

/// Parse session content as a SessionContainer
fn parse_session_content(tokens: &[Token]) -> Result<SessionContainer, BlockParseError> {
    if tokens.is_empty() {
        return Err(BlockParseError::InvalidStructure(
            "Session content cannot be empty".to_string(),
        ));
    }

    // Use the TokenTreeBuilder to correctly handle indentation
    let builder = crate::lexer::pipeline::TokenTreeBuilder::new();
    let token_tree = builder
        .build_tree(tokens.to_vec())
        .map_err(|e| BlockParseError::InvalidStructure(e.to_string()))?;

    // Use the block parser to parse the content
    let block_parser = crate::parser::pipeline::parse_blocks::BlockParser::new();
    let elements = block_parser.parse_blocks(token_tree)?;

    // Convert ElementNode to SessionContainerElement
    let content: Vec<SessionContainerElement> = elements
        .into_iter()
        .map(|element| match element {
            ElementNode::ParagraphBlock(paragraph) => {
                Ok(SessionContainerElement::Paragraph(paragraph))
            }
            ElementNode::SessionBlock(session) => Ok(SessionContainerElement::Session(session)),
            ElementNode::ListBlock(list) => Ok(SessionContainerElement::List(list)),
            ElementNode::DefinitionBlock(definition) => {
                Ok(SessionContainerElement::Definition(definition))
            }
            ElementNode::VerbatimBlock(verbatim) => Ok(SessionContainerElement::Verbatim(verbatim)),
            ElementNode::AnnotationBlock(annotation) => {
                Ok(SessionContainerElement::Annotation(annotation))
            }
            ElementNode::BlankLine(blank_line) => {
                Ok(SessionContainerElement::BlankLine(blank_line))
            }
            // TODO: Handle other element types as they're implemented
            _ => {
                // For now, skip unsupported elements
                Err(BlockParseError::InvalidStructure(format!(
                    "Unsupported element in session content: {:?}",
                    element
                )))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Create token sequence for the container
    let mut container_tokens = TokenSequence::new();
    container_tokens.tokens = tokens.to_vec();

    // Create the session container
    let container = SessionContainer {
        content,
        annotations: Vec::new(), // TODO: Parse annotations when implemented
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: container_tokens,
    };

    Ok(container)
}
