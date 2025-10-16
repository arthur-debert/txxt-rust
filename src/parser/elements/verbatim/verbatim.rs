//! # Verbatim Parser Module
//!
//! This module contains the logic for parsing verbatim blocks - the elements
//! that preserve exact formatting and spacing for code, preformatted text,
//! and other content requiring precise layout preservation.
//!
//! ## Overview
//!
//! Verbatim blocks preserve exact formatting and spacing for code, preformatted text,
//! and other content requiring precise layout preservation. They disable all inline
//! processing and maintain exact whitespace, line breaks, and character sequences.
//! Verbatim blocks support both in-flow and stretched modes for different layout needs.
//!
//! ## Grammar
//!
//! From [`docs/specs/core/syntax.txxt`]:
//!
//! ```text
//! <verbatim> = <verbatim-marker> <verbatim-content>
//! <verbatim-marker> = "```" <verbatim-title>? <line-break>
//! <verbatim-title> = <text-line>
//! <verbatim-content> = <verbatim-line>+ <verbatim-marker>
//! <verbatim-line> = <any-character>+ <line-break>
//! ```
//!
//! Verbatim blocks use triple backticks as markers and can contain any content
//! including other txxt elements without processing.
//!
//! ## AST Structure
//!
//! Post-parsing semantic representation:
//!
//! ```text
//! Verbatim AST:
//!     ├── VerbatimBlock
//!     │   ├── title: Option<String>
//!     │   ├── content: String
//!     │   ├── mode: VerbatimMode
//!         Annotations attached to this element, post parsing at assembly (during time aanotations are regular items in container)
//!     │   ├── annotations: Vec<Annotation>
//!     │   └── tokens: TokenSequence
//! ```
//!
//! Key structural properties:
//! - Content stored as raw string (no inline processing)
//! - Title preserved exactly as provided
//! - Mode indicates in-flow vs stretched layout
//! - Source tokens maintained for reconstruction
//!
//! ## AST Node Declaration
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//! pub struct VerbatimBlock {
//!     /// Optional verbatim title
//!     pub title: Option<String>,
//!     /// Raw verbatim content (no processing)
//!     pub content: String,
//!     /// Verbatim layout mode
//!     pub mode: VerbatimMode,
//!         Annotations attached to this verbatim block
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
//! - `txxt.core.spec.verbatim.valid.inflow-mode` - Verbatim in in-flow mode
//! - `txxt.core.spec.verbatim.valid.stretched-mode` - Verbatim in stretched mode
//! - `txxt.core.spec.verbatim.valid.no-title` - Verbatim without title
//! - `txxt.core.spec.verbatim.valid.with-title` - Verbatim with title
//!
//! ## Ensemble Examples
//!
//! From [`docs/specs/elements/verbatim/examples/`]:
//!
//! - `01-simple-nosession-single-verbatim.txxt` - Single verbatim only
//! - `02-simple-nosession-multiple-verbatim.txxt` - Multiple verbatim only
//! - `03-simple-flat-single-verbatim.txxt` - Single session with verbatim
//! - `04-simple-flat-multiple-verbatim.txxt` - Multiple sessions with verbatim
//! - `05-simple-nested-multiple-verbatim.txxt` - Nested sessions with verbatim
//!
//! ## AST Assertion Example
//!
//! ```rust,ignore
//! use tests::assertions::{assert_verbatim, VerbatimExpected};
//!
//! // Minimal validation (one property)
//! assert_verbatim(&element, VerbatimExpected {
//!     content_contains: Some("code"),
//!     ..Default::default()
//! });
//!
//! // Comprehensive validation (many properties)
//! assert_verbatim(&element, VerbatimExpected {
//!     title: Some("example"),
//!     content: Some("def hello():\n    print('world')"),
//!     mode: Some(VerbatimMode::InFlow),
//!     ..Default::default()
//! });
//! ```
//!
//! ## Processing Rules
//!
//! Verbatim blocks follow the standard processing pattern with these specific requirements:
//!
//! ### Recognition Criteria
//! - Start with triple backticks (```)
//! - Optional title on same line as opening marker
//! - Content continues until closing triple backticks
//! - No inline processing applied to content
//!
//! ### Verbatim Parsing
//! 1. Extract title from opening marker line
//! 2. Collect all content until closing marker
//! 3. Preserve exact whitespace and formatting
//! 4. Determine layout mode (in-flow vs stretched)
//!
//! ### Content Processing
//! 1. Store content as raw string (no processing)
//! 2. Preserve exact line breaks and indentation
//! 3. Maintain all whitespace and special characters
//! 4. Handle nested verbatim blocks as raw text
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/verbatim/verbatim.txxt`]
//! - **AST Node**: [`src/ast/elements/verbatim/verbatim.rs`]
//! - **Tokenizer**: [`src/lexer/elements/verbatim.rs`]
//! - **Test Assertions**: [`tests/assertions/elements/verbatim/`]
//! - **Corpora**: [`docs/dev/parser-core/per-element-corpora.txxt`]
//! - **Ensemble Examples**: [`docs/specs/elements/verbatim/examples/`]
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
//! ## Verbatim specificity:
//!
//!  Verbatim blocks are the only elements that completely disable inline processing,
//!  preserving exact formatting and spacing. They support two layout modes: in-flow
//!  (content indented +1 from title) and stretched (content at absolute column 2),
//!  enabling flexible code and preformatted text presentation.
//!
