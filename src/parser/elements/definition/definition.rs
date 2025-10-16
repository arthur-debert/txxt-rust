//! # Definition Parser Module
//!
//! This module contains the logic for parsing definitions - the structured elements
//! that define terms, concepts, and entities with their associated descriptions.
//!
//! ## Overview
//!
//! Definitions are structured elements that define terms, concepts, and entities with
//! their associated descriptions. They provide a semantic way to establish terminology,
//! create glossaries, and build knowledge bases. Definitions support both simple
//! term-description pairs and complex structured definitions with multiple components.
//!
//! ## Grammar
//!
//! From [`docs/specs/core/syntax.txxt`]:
//!
//! ```text
//! <definition> = <definition-marker> <definition-content>
//! <definition-marker> = "::" <whitespace> <definition-label> <whitespace> "::"
//! <definition-content> = <text-line>
//! ```
//!
//! Definitions use the `::` marker syntax similar to annotations but are semantically
//! distinct as they define rather than annotate content.
//!
//! ## AST Structure
//!
//! Post-parsing semantic representation:
//!
//! ```text
//! Definition AST:
//!     ├── DefinitionBlock
//!     │   ├── label: String
//!     │   ├── content: Vec<Inline>
//!         Annotations attached to this element, post parsing at assembly (during time aanotations are regular items in container)
//!     │   ├── annotations: Vec<Annotation>
//!     │   └── tokens: TokenSequence
//! ```
//!
//! Key structural properties:
//! - Label extracted from definition marker
//! - Content stored as inline elements (supports formatting)
//! - Self-referential structure (definitions can contain annotations)
//! - Source tokens maintained for reconstruction
//!
//! ## AST Node Declaration
//!
//! ```rust
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//! pub struct DefinitionBlock {
//!     /// Definition label/term
//!     pub label: String,
//!     /// Definition content with inline formatting
//!     pub content: Vec<Inline>,
//!         Annotations attached to this definition
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
//! - `txxt.core.spec.definition.valid.simple-term` - Basic definition with term
//! - `txxt.core.spec.definition.valid.multiline-description` - Definition with multiline description
//! - `txxt.core.spec.definition.valid.empty-description` - Definition with empty description
//! - `txxt.core.spec.definition.valid.nested-definition` - Definition containing another definition
//!
//! ## Ensemble Examples
//!
//! From [`docs/specs/elements/definition/examples/`]:
//!
//! - `01-simple-nosession-single-definition.txxt` - Single definition only
//! - `02-simple-nosession-multiple-definition.txxt` - Multiple definitions only
//! - `03-simple-flat-single-definition.txxt` - Single session with definition
//! - `04-simple-flat-multiple-definition.txxt` - Multiple sessions with definitions
//! - `05-simple-nested-multiple-definition.txxt` - Nested sessions with definitions
//!
//! ## AST Assertion Example
//!
//! ```rust
//! use tests::assertions::{assert_definition, DefinitionExpected};
//!
//! // Minimal validation (one property)
//! assert_definition(&element, DefinitionExpected {
//!     label: Some("term"),
//!     ..Default::default()
//! });
//!
//! // Comprehensive validation (many properties)
//! assert_definition(&element, DefinitionExpected {
//!     label: Some("concept"),
//!     content_contains: Some("explanation"),
//!     has_formatting: Some(false),
//!     ..Default::default()
//! });
//! ```
//!
//! ## Processing Rules
//!
//! Definitions follow the standard processing pattern with these specific requirements:
//!
//! ### Recognition Criteria
//! - Start with `::` marker
//! - Contain definition label between markers
//! - End with `::` marker
//! - Can contain any text content after markers
//!
//! ### Definition Parsing
//! 1. Extract label from between `::` markers
//! 2. Parse remaining content as inline elements
//! 3. Apply standard whitespace normalization
//! 4. Validate definition structure and content
//!
//! ### Content Processing
//! 1. Parse definition content through inline parser
//! 2. Support all inline formatting elements
//! 3. Preserve exact whitespace and formatting
//! 4. Handle nested definitions recursively
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/definition/definition.txxt`]
//! - **AST Node**: [`src/ast/elements/definition/definition.rs`]
//! - **Tokenizer**: [`src/lexer/elements/definition.rs`]
//! - **Test Assertions**: [`tests/assertions/elements/definition/`]
//! - **Corpora**: [`docs/dev/parser-core/per-element-corpora.txxt`]
//! - **Ensemble Examples**: [`docs/specs/elements/definition/examples/`]
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
//! ## Definition specificity:
//!
//!  Definitions are semantic elements that establish terminology and create
//!  knowledge bases. They use the same marker syntax as annotations but serve
//!  a distinct purpose in defining rather than annotating content, enabling
//!  automated glossary generation and semantic document analysis.
//!
