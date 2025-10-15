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

// All imports now handled by re-exports

// BREAKING CHANGE: Using new spec-aligned elements only
// All types now defined in elements/ module

// Re-export new types as the canonical implementations
pub use super::{
    definition::{DefinitionBlock as Definition, DefinitionTerm},
    list::{ListBlock as List, ListDecorationType, ListItem},
    verbatim::{VerbatimBlock, VerbatimType},
};

/// All block-level elements in TXXT documents
///
/// Blocks represent structural units that can contain other content.
/// Every block can have annotations attached based on proximity rules.
///
/// BREAKING CHANGE: Now uses new spec-aligned element types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Block {
    // Leaf blocks (cannot contain other blocks)
    Paragraph(super::paragraph::ParagraphBlock),
    VerbatimBlock(VerbatimBlock),
    BlankLine(super::core::BlankLine),

    // Content container blocks (cannot host sessions)
    List(List),
    Definition(Definition),

    // Session container blocks (can host new document sessions)
    Session(super::session::SessionBlock),

    // Container nodes for indented content
    Container(super::containers::ContentContainer),
}

// All block-level types now defined in elements/ - see re-exports above
// This maintains backward compatibility while using the new spec-aligned structure
