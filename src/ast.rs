//! AST module for TXXT format
//!
//! This module defines a comprehensive, type-safe AST structure for TXXT documents
//! that serves multiple tooling needs including language servers, formatters, linters,
//! and converters.
//!
//! # Core Design Principles
//!
//! ## Token-Level Precision Architecture
//!
//! The AST handles both tokens and parsed structure to facilitate language server features,
//! source mapping, and round-tripping. Every text element maintains character-level precision
//! through [`TokenSequence`] for:
//! - Hover information at exact cursor positions
//! - Autocomplete triggering within identifiers  
//! - Precise syntax highlighting and error underlining
//! - Perfect source reconstruction
//!
//! ## Container Indentation Pattern
//!
//! **Critical architectural insight**: The container is what gets indented, not the parent element.
//! This explains why flat lists don't need indentation - only nested content requires a [`Container`].
//!
//! Example:
//! ```txxt
//! - Item 1
//! - Item 2
//!   - Nested item    // This creates a Container
//! ```
//!
//! AST Structure:
//! ```text
//! List
//! ├── ListItem("Item 1")
//! ├── ListItem("Item 2")
//! └── Container
//!     └── List
//!         └── ListItem("Nested item")
//! ```
//!
//! ## Uniform Type System
//!
//! We enforce a homogeneous tree model where any element node can have parameters
//! or annotations. While some combinations don't occur in practice (language has no
//! syntactic construct for them), the type system remains uniform for consistency.
//!
//! ## Text Transform Layer
//!
//! Every piece of text goes through a uniform transform layer via [`TextTransform`]:
//! - `Identity(Text("banana"))` for plain text
//! - `Emphasis(vec![Identity(Text("important"))])` for *important*
//! - `Strong(vec![Emphasis(vec![Identity(Text("both"))])])` for **_both_**
//!
//! This provides consistent text handling across all contexts while maintaining
//! character-level precision for tooling.
//!
//! # Parsing Pipeline
//!
//! ## Phase 1: Lexer
//!
//! ### 1.a. Verbatim Line Marking
//! - **Stateful isolation**: Verbatim content is sacred and needs special handling
//! - **Critical insight**: This is the ONLY stateful part, keeping complexity contained
//! - **AST mapping**: Maps to [`VerbatimContent`] with exact preservation
//!
//! ### 1.b. Tokenization  
//! - **Token generation**: Produces character-precise tokens needed for language server
//! - **AST mapping**: Maps directly to [`Token`] enum with [`SourceSpan`] positioning
//!
//! ## Phase 2: Parser
//!
//! ### 2.a. Block Grouping
//! - **Indent/dedent processing**: Creates hierarchical structure using container pattern
//! - **Tree of token lists**: Perfect for the container indentation architecture
//! - **AST mapping**: Maps to [`Container`] nodes with proper nesting
//!
//! ### 2.b. Parsing
//! - **Token list → AST nodes**: Converts grouped tokens into semantic structure
//! - **Recursive processing**: Handles nested containers correctly
//! - **AST output**: Produces the rich type-safe AST defined in this module
//!
//! ## Phase 3: Post-Processing
//!
//! ### 3.a. Assembly (Not yet implemented)
//! - **Document metadata**: Parser version, file path, timestamps → [`AssemblyInfo`]
//! - **Annotation attachment**: Critical for the proximity-based annotation system
//! - **Final document**: Raw AST → fully assembled [`Document`]
//!
//! # Node Design Decision
//!
//! ## Custom Implementation with Pandoc Inspiration
//!
//! We chose a **custom AST implementation** rather than extending existing libraries because:
//!
//! ### Why Not Existing Libraries?
//! - **Markdown libraries** (pulldown-cmark, comrak) are optimized for CommonMark/GFM, not extensible text formats
//! - **Generic tree libraries** lack domain-specific semantics for text processing
//! - **Pandoc's Haskell AST** provides excellent design patterns but needs Rust-native implementation
//!
//! ### Our Approach: Rowan-Inspired Red-Green Trees
//! - **Red-green pattern**: Inspired by rowan (used by rust-analyzer) for efficient tree operations
//! - **Structural sharing**: Memory efficiency through shared immutable nodes  
//! - **Lossless representation**: Preserves all source information including whitespace
//! - **Incremental updates**: Foundation for future language server incremental parsing
//!
//! ### Type Safety Strategy
//! - **Enum-based nodes**: Compile-time verification of AST structure
//! - **Rich metadata**: [`Parameters`] and [`Annotation`] systems for extensibility
//! - **Token integration**: Character-precise positioning throughout the tree
//! - **Visitor patterns**: Type-safe traversal with exhaustive pattern matching
//!
//! ## Performance Characteristics
//! - **Memory efficient**: Structural sharing reduces duplication
//! - **Language server optimized**: Character-level precision without performance penalty
//! - **Incremental friendly**: Tree structure supports future incremental parsing
//! - **Tooling focused**: Rich metadata supports linters, formatters, converters
//!
//! # Module Organization
//!
//! The AST is organized into focused modules for maintainability:
//!
//! - [`annotations`] - Metadata attachment system with proximity rules
//! - [`base`] - Core document structure and assembly information  
//! - [`blocks`] - Block-level elements (verbatim, lists, definitions)
//! - [`inlines`] - Inline elements with text transform layer
//! - [`parameters`] - Shared metadata system (ref=, id=, severity=)
//! - [`reference_types`] - References and citations ([file.txxt], [@smith2023])
//! - [`structure`] - Hierarchical elements (containers, sessions, paragraphs)
//! - [`tokens`] - Character-precise positioning for language servers

// ============================================================================
// OLD AST SYSTEM (for current parser - active by default)
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple AST node for the TXXT parser (legacy)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode {
    /// The type of this AST node
    pub node_type: String,

    /// Child nodes
    pub children: Vec<AstNode>,

    /// Additional attributes/parameters for this node
    pub attributes: HashMap<String, String>,

    /// Text content for leaf nodes
    pub content: Option<String>,

    /// Source location information
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

impl AstNode {
    /// Create a new AST node with the given type
    pub fn new(node_type: String) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            attributes: HashMap::new(),
            content: None,
            start_line: None,
            end_line: None,
        }
    }

    /// Create a new AST node with type and content
    pub fn with_content(node_type: String, content: String) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            attributes: HashMap::new(),
            content: Some(content),
            start_line: None,
            end_line: None,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    /// Set an attribute
    pub fn set_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }

    /// Set source location
    pub fn set_location(&mut self, start_line: usize, end_line: usize) {
        self.start_line = Some(start_line);
        self.end_line = Some(end_line);
    }
}

/// Container for the entire parsed document (legacy)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Root AST node
    pub root: AstNode,

    /// Source file path
    pub source: String,
}

impl Document {
    pub fn new(source: String) -> Self {
        Self {
            root: AstNode::new("document".to_string()),
            source,
        }
    }
}

// ============================================================================
// NEW AST SYSTEM (disabled by default, enable with --features new-ast)
// ============================================================================

#[cfg(feature = "new-ast")]
pub mod annotations;
#[cfg(feature = "new-ast")]
pub mod base;
#[cfg(feature = "new-ast")]
pub mod blocks;
#[cfg(feature = "new-ast")]
pub mod inlines;
#[cfg(feature = "new-ast")]
pub mod parameters;
#[cfg(feature = "new-ast")]
pub mod reference_types;
#[cfg(feature = "new-ast")]
pub mod structure;
#[cfg(feature = "new-ast")]
pub mod tokens;
