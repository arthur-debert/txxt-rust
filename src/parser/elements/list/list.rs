//! # List Parser Module
//!
//! This module contains the logic for parsing lists - the structured elements
//! that organize content into ordered and unordered sequences with various
//! decoration styles and nesting capabilities.
//!
//! ## Overview
//!
//! Lists are structured elements that organize content into ordered and unordered
//! sequences with various decoration styles and nesting capabilities. They provide
//! semantic organization for sequential content, enable automated numbering,
//! and support complex nested structures with different decoration styles
//! at each level.
//!
//! ## Grammar
//!
//! From [`docs/specs/core/syntax.txxt`]:
//!
//! ```text
//! <list> = <list-item>+
//! <list-item> = <sequence-marker> <whitespace> <list-content>
//! <sequence-marker> = <plain-marker> | <numerical-marker> | <alphabetical-marker> | <roman-marker>
//! <list-content> = <text-line> | <indented-block>
//! ```
//!
//! Lists use sequence markers to identify items and support various decoration
//! styles including plain, numerical, alphabetical, and roman numbering.
//!
//! ## AST Structure
//!
//! Post-parsing semantic representation:
//!
//! ```text
//! List AST:
//!     ├── ListBlock
//!     │   ├── items: Vec<ListItem>
//!     │   │   ├── marker: SequenceMarker
//!     │   │   ├── content: Vec<Inline>
//!     │   │   └── tokens: ScannerTokenSequence
//!     │   ├── decoration: ListDecoration
//!         Annotations attached to this element, post parsing at assembly (during time aanotations are regular items in container)
//!     │   ├── annotations: Vec<Annotation>
//!     │   └── tokens: ScannerTokenSequence
//! ```
//!
//! Key structural properties:
//! - Items stored as sequence of list items
//! - Each item contains marker and content
//! - Decoration style determined from markers
//! - Source tokens maintained for reconstruction
//!
//! ## AST Node Declaration
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//! pub struct ListBlock {
//!     /// List items with markers and content
//!     pub items: Vec<ListItem>,
//!     /// List decoration style
//!     pub decoration: ListDecoration,
//!         Annotations attached to this list
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
//! - `txxt.core.spec.list.valid.plain-decoration` - List with plain decoration
//! - `txxt.core.spec.list.valid.numerical-decoration` - List with numerical decoration
//! - `txxt.core.spec.list.valid.alphabetical-decoration` - List with alphabetical decoration
//! - `txxt.core.spec.list.valid.roman-decoration` - List with roman decoration
//!
//! ## Ensemble Examples
//!
//! From [`docs/specs/elements/list/examples/`]:
//!
//! - `01-simple-nosession-single-list.txxt` - Single list only
//! - `02-simple-nosession-multiple-list.txxt` - Multiple lists only
//! - `03-simple-flat-single-list.txxt` - Single session with list
//! - `04-simple-flat-multiple-list.txxt` - Multiple sessions with lists
//! - `05-simple-nested-multiple-list.txxt` - Nested sessions with lists
//!
//! ## AST Assertion Example
//!
//! ```rust,ignore
//! use tests::assertions::{assert_list, ListExpected};
//!
//! // Minimal validation (one property)
//! assert_list(&element, ListExpected {
//!     item_count: Some(3),
//!     ..Default::default()
//! });
//!
//! // Comprehensive validation (many properties)
//! assert_list(&element, ListExpected {
//!     decoration: Some(ListDecoration::Numerical),
//!     item_count: Some(3),
//!     first_item_contains: Some("first"),
//!     ..Default::default()
//! });
//! ```
//!
//! ## Processing Rules
//!
//! Lists follow the standard processing pattern with these specific requirements:
//!
//! ### Recognition Criteria
//! - Start with sequence marker (plain, numerical, alphabetical, roman)
//! - Followed by whitespace and content
//! - Multiple consecutive items form a list
//! - Different decoration styles can be mixed
//!
//! ### List Parsing
//! 1. Identify sequence markers and decoration styles
//! 2. Group consecutive items with same decoration
//! 3. Parse item content through inline parser
//! 4. Handle mixed decoration styles appropriately
//!
//! ### Content Processing
//! 1. Parse item content through inline parser
//! 2. Support all inline formatting elements
//! 3. Preserve exact whitespace and formatting
//! 4. Handle nested lists and other block elements
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/list/list.txxt`]
//! - **AST Node**: [`src/ast/elements/list/list.rs`]
//! - **Tokenizer**: [`src/lexer/elements/list.rs`]
//! - **Test Assertions**: [`tests/assertions/elements/list/`]
//! - **Corpora**: [`docs/dev/parser-core/per-element-corpora.txxt`]
//! - **Ensemble Examples**: [`docs/specs/elements/list/examples/`]
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
//! ## List specificity:
//!
//!  Lists support multiple decoration styles (plain, numerical, alphabetical, roman)
//!  and can mix different styles within the same list. They provide semantic
//!  organization for sequential content while maintaining flexible formatting
//!  and enabling automated numbering and cross-referencing.
//!
