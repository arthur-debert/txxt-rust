//! Block-level elements: verbatim, lists, definitions, and annotations
//!
//! This module defines the content container blocks that structure document
//! content but cannot host new document sessions (unlike sessions).
//!
//! # Parsing Pipeline Position
//!
//! **Phase 1.a: Verbatim Line Marking** (VerbatimBlock)
//! **Phase 2.b: Parsing** (Lists, Definitions, Block AST)
//!
//! Verbatim blocks are the only stateful part of parsing - they're identified and
//! marked during the initial lexer pass to prevent their content from being processed
//! as TXXT. All other blocks are parsed during the main parsing phase from grouped tokens.
//!
//! Pipeline: `Source` → **`Verbatim Marking`** → `Tokens` → `Grouping` → **`Block Parsing`** → `Assembly`
//!
//! ## Verbatim Handling (1.a)
//!
//! Verbatim blocks require special stateful processing during lexing because their
//! content is sacred and must not be processed as TXXT syntax. This is the ONLY
//! stateful part of the entire parsing pipeline, keeping complexity isolated.
//!
//! ## Block Parsing (2.b)
//!
//! Lists, definitions, and other blocks are parsed from grouped token lists into
//! rich semantic structures with sophisticated styling and metadata support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    annotations::Annotation,
    inlines::Inline,
    parameters::Parameters,
    structure::{Container, NumberingForm, NumberingStyle},
    tokens::TokenSequence,
};

/// All block-level elements in TXXT documents
///
/// Blocks represent structural units that can contain other content.
/// Every block can have annotations attached based on proximity rules.
///
/// Key distinction:
/// - Content containers (List, Definition, etc.): Cannot host sessions
/// - Session containers (Session): Can host new document sessions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Block {
    // Leaf blocks (cannot contain other blocks)
    Paragraph(super::structure::Paragraph),
    VerbatimBlock(VerbatimBlock),
    BlankLine(super::structure::BlankLine),

    // Content container blocks (cannot host sessions)
    List(List),
    Definition(Definition),

    // Session container blocks (can host new document sessions)
    Session(super::structure::Session),

    // The key architectural insight: explicit Container nodes for indented content
    Container(Container),
}

/// Verbatim block - content that bypasses all TXXT parsing
///
/// Verbatim blocks preserve their content exactly as written, without any
/// TXXT processing. They're essential for including code, configuration,
/// or other formats within TXXT documents.
///
/// Two types exist:
/// - In-flow: Integrated with regular content flow
/// - Stretched: Separate block with clear boundaries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbatimBlock {
    /// Raw content, no parsing or transforms applied  
    /// This content is sacred - no modification allowed
    pub raw: String,

    /// Type of verbatim block (in-flow vs stretched)
    pub verbatim_type: VerbatimType,

    /// Optional format hint (e.g., "rust", "json", "html")
    /// Used by syntax highlighters and other tooling
    pub format_hint: Option<String>,

    /// Parameters from verbatim block declaration
    /// Supports arbitrary key-value metadata including ref= for named anchors
    pub parameters: Parameters,

    /// Annotations attached to this verbatim block
    pub annotations: Vec<Annotation>,

    /// Raw tokens for source reconstruction
    pub tokens: TokenSequence,
}

/// Types of verbatim blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerbatimType {
    /// In-flow verbatim (inline with regular content)
    InFlow,

    /// Stretched verbatim (separate block with clear boundaries)
    Stretched,
}

/// List block with sophisticated styling support
///
/// Lists in TXXT support complex styling and numbering schemes commonly
/// needed in technical documentation. The key insights:
///
/// 1. **Styling is a list attribute, not item attribute**
///    - First item determines style for whole list
///    - Mixed styling is preserved but not validated
///    - Renderer can auto-correct if needed
///
/// 2. **Markers are preserved exactly**
///    - Parser saves actual input markers ("1.", "c)", "ii.", etc.)
///    - No validation of sequence or correctness
///    - Enables flexible authoring and automated correction
///
/// 3. **Forgiving parsing**
///    - Mixed styles don't cause errors
///    - Out-of-order numbering is accepted
///    - Content is preserved, inconsistencies noted
///
/// Examples:
/// ```txxt
/// 1. Mom          // Numerical list, short form
/// 2. Dad
///
/// a) Red          // Alphabetical list  
/// b) Blue
///
/// i) First        // Roman numeral list
/// ii) Second
///
/// 3. Wrong        // Inconsistent but parsed
/// a) Mixed        // Different style but preserved
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct List {
    /// Decoration/styling for this list (from first item)
    pub decoration_type: ListDecorationType,

    /// List items with their original markers
    pub items: Vec<ListItem>,

    /// Annotations attached to this list
    pub annotations: Vec<Annotation>,

    /// Raw tokens for source reconstruction
    pub tokens: TokenSequence,
}

/// List decoration/styling information
///
/// Combines numbering style with form to handle complex technical document
/// numbering like "1.a.i." (full form) vs "i." (short form).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListDecorationType {
    /// The numbering/marker style (plain, numerical, alphabetical, roman)
    pub style: NumberingStyle,

    /// Short form (1.) vs full form (1.a.i.)
    pub form: NumberingForm,
}

/// Individual list item with preserved marker
///
/// List items maintain both their semantic content and their original
/// marker text. This enables:
/// - Exact source reconstruction
/// - Style consistency checking
/// - Automated renumbering while preserving intent
/// - Flexible rendering options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    /// The actual marker text as it appears in source ("1.", "c)", "ii.", etc.)
    /// Preserved exactly for source reconstruction and style analysis
    pub marker: String,

    /// List item content (inline elements)
    pub content: Vec<Inline>,

    /// Nested content (if any) goes in a Container
    /// Following the indentation pattern: containers get indented, not parents
    pub nested: Option<Container>,

    /// Annotations attached to this specific list item
    pub annotations: Vec<Annotation>,

    /// Raw tokens for precise reconstruction
    pub tokens: TokenSequence,
}

/// Definition block - term and definition pairs
///
/// Definitions follow the container indentation pattern:
/// ```txxt
/// Term:               // DefinitionTerm (level 0)
///
///     Definition      // Container (level 1) -> definition content
/// ```
///
/// The term is not indented, but its definition content is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    /// The term being defined
    pub term: DefinitionTerm,

    /// Definition content (indented)
    pub content: Container,

    /// Parameters for metadata including ref= for named anchors and category= for organization
    pub parameters: Parameters,

    /// Annotations attached to this definition
    pub annotations: Vec<Annotation>,

    /// Raw tokens for source reconstruction
    pub tokens: TokenSequence,
}

/// Term part of a definition
///
/// The term appears at the base level, followed by a colon and then
/// indented definition content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionTerm {
    /// Term content (inline elements for formatting support)
    pub content: Vec<Inline>,

    /// Raw tokens for exact positioning
    pub tokens: TokenSequence,
}

impl Default for ListDecorationType {
    fn default() -> Self {
        Self {
            style: NumberingStyle::Plain,
            form: NumberingForm::Short,
        }
    }
}
