//! Powerful AST Query and Traversal API
//!
//! This module provides a comprehensive, ergonomic API for querying and traversing
//! TXXT AST nodes. The design is inspired by XPath and Django QuerySets while
//! maintaining idiomatic Rust patterns with compile-time safety.
//!
//! # Design Philosophy
//!
//! ## Type-Safe Traversal
//! - Compile-time guarantees for valid queries
//! - Zero-cost abstractions using iterators
//! - Builder pattern for readable, chainable queries
//!
//! ## Unist Compatibility
//! - Fully compatible with [Unist](https://github.com/syntax-tree/unist) specification
//! - Supports standard tree traversal patterns (preorder, postorder, breadth-first)
//! - Provides parent-child relationships and positional information
//! - Enables interoperability with the broader syntax-tree ecosystem
//!
//! # Quick Start
//!
//! ```rust
//! use txxt::ast::query::Query;
//!
//! # fn example(root: &txxt::ast::nodes::paragraph::Paragraph) {
//! // Find all paragraphs with specific parameters
//! let results = Query::from(root)
//!     .filter_type("Paragraph")
//!     .filter(|node| node.has_parameter("id"))
//!     .collect::<Vec<_>>();
//!
//! // Django QuerySet-like chaining
//! let annotated = Query::from(root)
//!     .descendants()
//!     .filter_type("Paragraph")
//!     .filter(|n| !n.annotations().is_empty())
//!     .first();
//!
//! // XPath-like selection
//! let nested_lists = Query::from(root)
//!     .select("List/ListItem/Container/List")
//!     .collect::<Vec<_>>();
//!
//! // Complex filtering with multiple criteria
//! let specific_nodes = Query::from(root)
//!     .descendants()
//!     .filter_type("Paragraph")
//!     .filter_parameter("severity", "error")
//!     .filter_content_contains("TODO")
//!     .collect::<Vec<_>>();
//! # }
//! ```
//!
//! # Core Concepts
//!
//! ## NodeRef - Universal Node Reference
//!
//! All nodes in the AST are accessed through [`NodeRef`], a unified wrapper that
//! provides common operations regardless of the underlying node type:
//!
//! ```rust
//! # use txxt::ast::query::NodeRef;
//! # fn example(node: NodeRef) {
//! // Access node type
//! let type_name = node.node_type();
//!
//! // Navigate relationships
//! if let Some(parent) = node.parent() {
//!     println!("Parent: {}", parent.node_type());
//! }
//!
//! // Get children (Unist-compatible)
//! for child in node.children() {
//!     println!("Child: {}", child.node_type());
//! }
//!
//! // Get content (text representation)
//! let text = node.text_content();
//! # }
//! ```
//!
//! ## Query Builder - Fluent API
//!
//! The [`Query`] builder provides a chainable interface for constructing complex queries:
//!
//! ```rust
//! # use txxt::ast::query::Query;
//! # fn example(root: &txxt::ast::nodes::paragraph::Paragraph) {
//! Query::from(root)
//!     .descendants()              // Traverse all descendants
//!     .filter_type("List")        // Only List nodes
//!     .filter(|n| n.depth() > 2)  // Nested more than 2 levels
//!     .map(|n| n.node_type())     // Transform to type names
//!     .collect::<Vec<_>>();       // Collect results
//! # }
//! ```
//!
//! ## Traversal Modes
//!
//! Multiple traversal strategies are supported, following Unist conventions:
//!
//! ```rust
//! # use txxt::ast::query::{Query, TraversalMode};
//! # fn example(root: &txxt::ast::nodes::paragraph::Paragraph) {
//! // Depth-first preorder (default, Unist NLR)
//! Query::from(root).descendants().collect::<Vec<_>>();
//!
//! // Depth-first postorder (Unist LRN)
//! Query::from(root).descendants_postorder().collect::<Vec<_>>();
//!
//! // Breadth-first
//! Query::from(root).descendants_breadth_first().collect::<Vec<_>>();
//!
//! // Direct children only
//! Query::from(root).children().collect::<Vec<_>>();
//! # }
//! ```
//!
//! # Advanced Usage
//!
//! ## Path-Based Selection (XPath-like)
//!
//! Select nodes using path expressions:
//!
//! ```rust
//! # use txxt::ast::query::Query;
//! # fn example(root: &txxt::ast::nodes::paragraph::Paragraph) {
//! // Select all lists
//! Query::from(root).select("//List").collect::<Vec<_>>();
//!
//! // Select nested lists
//! Query::from(root).select("List//List").collect::<Vec<_>>();
//!
//! // Select with conditions
//! Query::from(root)
//!     .select("//Paragraph")
//!     .filter_parameter("id", "intro")
//!     .collect::<Vec<_>>();
//! # }
//! ```
//!
//! ## Custom Predicates
//!
//! Filter using arbitrary predicates:
//!
//! ```rust
//! # use txxt::ast::query::Query;
//! # fn example(root: &txxt::ast::nodes::paragraph::Paragraph) {
//! // Find nodes matching complex criteria
//! let results = Query::from(root)
//!     .descendants()
//!     .filter(|node| {
//!         node.node_type() == "Paragraph" &&
//!         node.has_parameter("severity") &&
//!         node.text_content().len() > 100
//!     })
//!     .collect::<Vec<_>>();
//! # }
//! ```
//!
//! ## Sibling Navigation
//!
//! Navigate between siblings (Unist-compatible):
//!
//! ```rust
//! # use txxt::ast::query::NodeRef;
//! # fn example(node: NodeRef) {
//! // Next sibling
//! if let Some(next) = node.next_sibling() {
//!     println!("Next: {}", next.node_type());
//! }
//!
//! // Previous sibling
//! if let Some(prev) = node.previous_sibling() {
//!     println!("Previous: {}", prev.node_type());
//! }
//!
//! // All siblings
//! for sibling in node.siblings() {
//!     println!("Sibling: {}", sibling.node_type());
//! }
//! # }
//! ```
//!
//! ## Positional Information (Unist Position)
//!
//! Access source location information following Unist conventions:
//!
//! ```rust
//! # use txxt::ast::query::NodeRef;
//! # fn example(node: NodeRef) {
//! if let Some(pos) = node.position() {
//!     println!(
//!         "Node at line {}, column {} (offset {})",
//!         pos.start.line,
//!         pos.start.column,
//!         pos.start.offset
//!     );
//! }
//! # }
//! ```
//!
//! # Unist Compatibility
//!
//! This module implements the [Unist specification](https://github.com/syntax-tree/unist)
//! for universal syntax trees, enabling interoperability with tools from the
//! syntax-tree ecosystem (mdast, hast, nlcst, xast).
//!
//! ## Unist Node Mapping
//!
//! TXXT nodes map cleanly to Unist concepts:
//!
//! | TXXT Concept       | Unist Interface | Notes                                    |
//! |--------------------|-----------------|------------------------------------------|
//! | All AST nodes      | `Node`          | type, data, position                     |
//! | Container nodes    | `Parent`        | Has `children` array                     |
//! | Text content       | `Literal`       | Leaf nodes with `value`                  |
//! | ScannerTokenSequence      | `Position`      | Source location with start/end Points    |
//! | Parameters         | `data`          | Custom metadata per ecosystem            |
//!
//! ## Interoperability Benefits
//!
//! Unist compatibility provides:
//!
//! - **Standard Utilities**: Use existing unist-util-* packages for common operations
//! - **Tree Transformations**: Convert between TXXT and other formats (Markdown, HTML, etc.)
//! - **Ecosystem Tools**: Leverage syntax-tree tooling (linters, formatters, analyzers)
//! - **Specification Clarity**: Well-defined traversal and node relationship semantics
//!
//! ## Implementation Strategy
//!
//! We provide Unist compatibility through:
//!
//! 1. **Adapter Layer**: [`NodeRef`] implements Unist Node interface
//! 2. **Position Mapping**: ScannerTokenSequence maps to Unist Position/Point
//! 3. **Type Safety**: Rust enums provide stronger guarantees than Unist's type strings
//! 4. **Zero Cost**: Adapter is compile-time only, no runtime overhead
//!
//! ## Why Unist is a Natural Fit
//!
//! Unist compatibility aligns perfectly with TXXT's design:
//!
//! ### ✅ Shared Design Principles
//!
//! - **Lossless Representation**: Both preserve all source information
//! - **Position Tracking**: Character-level precision for tooling
//! - **Type Safety**: Strong node type guarantees (Rust enums vs. TypeScript)
//! - **Parent-Child Trees**: Standard hierarchical structure
//!
//! ### ✅ TXXT-Specific Benefits
//!
//! - **Text Format Focus**: Unist designed for text/markup (markdown, XML, natural language)
//! - **Extensibility**: Unist explicitly designed to be extended for domain-specific formats
//! - **Rich Inline Content**: Inline elements map naturally to Unist Literal nodes
//! - **Block Structure**: Containers and sessions are standard Parent nodes
//!
//! ### ✅ Practical Advantages
//!
//! - **Proven Ecosystem**: 30M+ monthly downloads of unist utilities in JS
//! - **Markdown Interop**: Natural bridge to mdast for Markdown conversion
//! - **Language Server**: Unist patterns already used in language servers (rust-analyzer uses similar concepts)
//! - **Documentation**: Well-specified behavior for edge cases
//!
//! ### ⚠️ Considerations
//!
//! While Unist is predominantly JavaScript/Node.js ecosystem, the specification is
//! language-agnostic. The core concepts (Parent, Literal, Position) are universal
//! and map cleanly to Rust types. We don't need JavaScript interop to benefit from
//! the standardized semantics and design patterns.
//!
//! ### 🔄 Cross-Format Conversion
//!
//! Unist compatibility enables seamless format conversion:
//!
//! ```rust,ignore
//! // TXXT → Unist → mdast (Markdown)
//! let txxt_ast = parse_txxt(source)?;
//! let unist_node = NodeRef::from(&txxt_ast);
//! let markdown = convert_unist_to_mdast(unist_node)?;
//!
//! // Unist → TXXT (import from Markdown/HTML)
//! let mdast = parse_markdown(md_source)?;
//! let unist_node = NodeRef::from_mdast(mdast);
//! let txxt_ast = convert_to_txxt(unist_node)?;
//! ```
//!
//! # Performance Characteristics
//!
//! The query API is designed for performance:
//!
//! - **Zero-Copy**: [`NodeRef`] borrows nodes, no cloning
//! - **Lazy Evaluation**: Iterator chains only compute when needed
//! - **Compiled Paths**: XPath-like expressions compiled to efficient filters
//! - **Cache-Friendly**: Breadth-first uses VecDeque, depth-first uses stack
//!
//! # Error Handling
//!
//! Queries use `Option` and `Result` idiomatically:
//!
//! ```rust
//! # use txxt::ast::query::Query;
//! # fn example(root: &txxt::ast::nodes::paragraph::Paragraph) {
//! // Option for nullable results
//! let first = Query::from(root).descendants().first();
//!
//! // Result for operations that can fail
//! let result = Query::from(root)
//!     .select("//InvalidPath")
//!     .map_err(|e| format!("Query failed: {}", e));
//! # }
//! ```
//!
//! # Testing
//!
//! The query module integrates with TXXT's corpus-based testing:
//!
//! ```rust,ignore
//! use tests::corpora::{TxxtCorpora, ProcessingStage};
//!
//! let corpus = TxxtCorpora::load_with_processing(
//!     "txxt.core.spec.paragraph.valid.multiline",
//!     ProcessingStage::ParsedAst
//! )?;
//!
//! let ast = corpus.ast().unwrap();
//! let paragraphs = Query::from(ast)
//!     .filter_type("Paragraph")
//!     .collect::<Vec<_>>();
//!
//! assert_eq!(paragraphs.len(), 2);
//! ```

pub mod builder;
pub mod node_ref;
pub mod predicates;
pub mod traversal;
pub mod unist;

pub use builder::Query;
pub use node_ref::NodeRef;
pub use predicates::{Predicate, PredicateExt};
pub use traversal::{TraversalMode, TreeIterator};
pub use unist::{UnistNode, UnistParent, UnistLiteral, Position, Point};
