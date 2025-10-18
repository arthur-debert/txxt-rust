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
//!     │   │   └── tokens: ScannerTokenSequence
//!     │   ├── content: SessionContainer
//!     │   │   └── content: Vec<Block>
//!     │   ├── annotations: Vec<Annotation>
//!     │   └── tokens: ScannerTokenSequence
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
//!     pub tokens: ScannerTokenSequence,
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

// TODO: Phase 2.b AST Construction - implement session parsing
// This will convert SemanticTokenList to SessionBlock AST elements
