//! Document parser - main entry point for Phase 2 parsing
//!
//! TODO: This is a stub implementation. The real parser will:
//! 1. Take new Token enum variants from the tokenizer
//! 2. Convert to proper typed AST nodes using new AST system
//! 3. Follow the finalized specs in docs/specs/core/

use crate::ast::base::Document;

/// Main parser that converts Tokens to AST
///
/// TODO: Rewrite to use new Token enum and typed AST nodes
pub struct DocumentParser {
    source: String,
}

impl DocumentParser {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    /// Parse tokens into an AST Document
    ///
    /// TODO: Complete rewrite needed for new architecture:
    /// - Process Vec<Token> from new tokenizer
    /// - Implement block grouping (Phase 2a)
    /// - Convert to typed AST nodes (Phase 2b)
    pub fn parse(&self, _tokens: &[crate::ast::tokens::Token]) -> Document {
        // STUB: Return minimal document until real implementation
        use crate::ast::{
            base::{AssemblyInfo, Meta},
            structure::{Container, ContainerType},
        };

        Document {
            meta: Meta::default(),
            content: Container {
                container_type: ContainerType::Session,
                content: Vec::new(),
                annotations: Vec::new(),
            },
            assembly_info: AssemblyInfo {
                parser_version: env!("CARGO_PKG_VERSION").to_string(),
                source_path: Some(self.source.clone()),
                processed_at: None,
                stats: Default::default(),
            },
        }
    }
}

/// Main entry point for parsing
///
/// TODO: Rewrite to use new Token enum and phase 2 architecture
pub fn parse_document(source: String, tokens: &[crate::ast::tokens::Token]) -> Document {
    let parser = DocumentParser::new(source);
    parser.parse(tokens)
}
