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
//!         being used yet, that is approach it as work in progress and be mindful that 
//!         the AST assertions can have bugs or gaps, in which case verify through
//!         the ast testing and then fix / improve the AST assertions.
//!  3. Use the ensemble examples to test the parser for the corpora examples.
//!  3. Use the corpora stack to test exception / errors.
//! 
//! ## Paragraph specificity: 
//! 
//!  Paragraphs are the catchall element, that is parsing a pargraph should never fail, as 
//!  long as we have a line with characters.
//! 
