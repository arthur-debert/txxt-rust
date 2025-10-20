#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
#![allow(clippy::doc_overindented_list_items)]

//! TXXT Parser and Processor
//!
//! ============================================================================
//! ARCHITECTURE OVERVIEW
//! ============================================================================
//!
//! TXXT processing transforms plain text documents into structured representations
//! through a three-phase pipeline. Each phase has distinct responsibilities and
//! produces well-defined intermediate representations.
//!
//!
//! TERMINOLOGY
//!
//! Understanding the processing model requires clear terminology:
//!
//! - Phase: One of three high-level processing stages
//!     · Phase 1: Lexer
//!     · Phase 2: Parser
//!     · Phase 3: Assembler
//!
//! - Step: Sub-operations within a phase
//!     · Example: Semantic analysis, AST construction, inline parsing
//!     · Each step transforms data and passes it to the next step
//!
//! - Stage: CLI and test concept for where to stop processing
//!     · Used for inspection, debugging, and intermediate output
//!     · Examples: scanner-tokens, semantic-tokens, ast-block, ast-full
//!     · Stages map to specific steps within phases
//!
//!
//! DATA FLOW
//!
//! The processing pipeline transforms data through these representations:
//!
//! ```text
//! Source Text (String)
//!     ↓
//! Phase 1: Lexer
//!     ↓
//! ScannerTokenTree
//!     ↓
//! Phase 2: Parser
//!     ↓
//! Vec<ElementNode>
//!     ↓
//! Phase 3: Assembler
//!     ↓
//! Document
//! ```
//!
//!
//! PHASE 1: LEXER
//!
//! Converts source text into hierarchical token structures.
//!
//! Module: [`lexer`]
//! Entry point: [`process::process_lexer`]
//!
//! Processing steps:
//!
//! - Step 1.a: Verbatim scanning
//!     Purpose: Identify and mark verbatim regions
//!     Input: Raw source text
//!     Output: Text with verbatim boundaries marked
//!     Note: Handled internally by tokenize function
//!
//! - Step 1.b: Tokenization
//!     Purpose: Convert text to flat token stream
//!     Input: Source text with verbatim markers
//!     Output: Vec<ScannerToken>
//!     Module: [`lexer::tokenize`]
//!     Produces: Low-level tokens with precise source positions
//!
//! - Step 1.c: Token tree building
//!     Purpose: Organize flat tokens into hierarchical structure
//!     Input: Vec<ScannerToken>
//!     Output: ScannerTokenTree
//!     Module: [`lexer::token_tree_builder`]
//!     Produces: Nested token structure respecting indentation
//!
//!
//! PHASE 2: PARSER
//!
//! Converts tokens into Abstract Syntax Tree nodes.
//!
//! Module: [`parser`]
//! Entry point: [`process::process_parser`]
//!
//! Processing steps:
//!
//! - Step 2.a: Semantic analysis
//!     Purpose: Analyze tokens and produce semantic tokens
//!     Input: Vec<ScannerToken>
//!     Output: SemanticTokenList
//!     Module: [`parser::semantic_analysis`]
//!     Produces: Tokens with semantic meaning attached
//!
//! - Step 2.b: AST construction
//!     Purpose: Build AST tree from semantic tokens
//!     Input: SemanticTokenList
//!     Output: Vec<ElementNode>
//!     Module: [`parser::ast_construction`]
//!     Produces: Block-level AST structure
//!
//! - Step 2.c: Inline parsing
//!     Purpose: Parse inline formatting within text content
//!     Input: Vec<ElementNode> with unparsed inline text
//!     Output: Vec<ElementNode> with parsed inline elements
//!     Module: [`parser::inline_parsing`]
//!     Produces: Complete AST with formatting, references, etc.
//!
//!
//! PHASE 3: ASSEMBLER
//!
//! Converts AST nodes into final document structure.
//!
//! Module: [`assembler`]
//! Entry point: [`process::process_assembler`]
//!
//! Processing steps:
//!
//! - Step 3.a: Document assembly
//!     Purpose: Wrap AST elements in document structure
//!     Input: Vec<ElementNode>
//!     Output: Document with metadata
//!     Module: [`assembler::document_assembly`]
//!     Produces: Document with assembly info, stats, metadata
//!
//! - Step 3.b: Annotation attachment
//!     Purpose: Attach annotations to target elements
//!     Input: Document with unattached annotations
//!     Output: Document with annotations properly attached
//!     Module: [`assembler::annotation_attachment`]
//!     Produces: Final document with proximity-based annotation attachment
//!
//!
//! CLI STAGES
//!
//! The CLI exposes inspection points called stages that map to processing steps:
//!
//! - scanner-tokens
//!     Maps to: Phase 1, Step 1.b output
//!     Data structure: Vec<ScannerToken>
//!     Use case: Debug tokenization
//!
//! - semantic-tokens
//!     Maps to: Phase 2, Step 2.a output
//!     Data structure: SemanticTokenList
//!     Use case: Debug semantic analysis
//!
//! - ast-block
//!     Maps to: Phase 2, Step 2.b output
//!     Data structure: Vec<ElementNode> (no inline parsing)
//!     Use case: Debug block-level parsing
//!
//! - ast-inlines
//!     Maps to: Phase 2, Step 2.c output
//!     Data structure: Vec<ElementNode> (with inline parsing)
//!     Use case: Debug inline element parsing
//!
//! - ast-document
//!     Maps to: Phase 3, Step 3.a output
//!     Data structure: Document (before annotation attachment)
//!     Use case: Debug document assembly
//!
//! - ast-full
//!     Maps to: Phase 3, Step 3.b output (complete)
//!     Data structure: Document (final)
//!     Use case: Production output, final validation
//!
//!
//! MODULE ORGANIZATION
//!
//! Implementation modules mirror the phase structure:
//!
//! - [`lexer`]: Phase 1 implementation
//!     · Core tokenization logic
//!     · Token tree building
//!     · Element-specific lexers
//!
//! - [`parser`]: Phase 2 implementation
//!     · Semantic analysis
//!     · AST construction
//!     · Inline parsing
//!     · Element-specific parsers
//!
//! - [`assembler`]: Phase 3 implementation
//!     · Document assembly
//!     · Annotation attachment
//!
//! - [`process`]: Top-level orchestration
//!     · process_lexer: Execute Phase 1
//!     · process_parser: Execute Phase 2
//!     · process_assembler: Execute Phase 3
//!     · process_full: Execute all three phases
//!
//! - [`ast`]: AST node definitions
//!     · Element type definitions
//!     · Tree traversal utilities
//!     · Semantic token types
//!
//! - [`api`]: Pure processing functions
//!     · No I/O or side effects
//!     · Used by CLI and tests
//!     · Stage-to-output conversion
//!
//! - [`processing_stages`]: CLI stage registry
//!     · Stage definitions
//!     · Format definitions
//!     · Stage-format compatibility
//!
//!
//! USAGE EXAMPLES
//!
//! Process complete document:
//!
//! ```
//! use txxt::process::process_full;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = "This is a TXXT document.";
//! let document = process_full(source, Some("doc.txxt".to_string()))?;
//! # Ok(())
//! # }
//! ```
//!
//! Process to specific phase:
//!
//! ```
//! use txxt::process::{process_lexer, process_parser};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let source = "This is a TXXT document.";
//! let tokens = process_lexer(source)?;
//! let ast = process_parser(tokens)?;
//! # Ok(())
//! # }
//! ```
//!
//! Access individual components:
//!
//! ```
//! use txxt::lexer::tokenize;
//! use txxt::parser::SemanticAnalyzer;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let source = "This is a TXXT document.";
//! let tokens = tokenize(source);
//! let analyzer = SemanticAnalyzer::new();
//! let semantic_tokens = analyzer.analyze(tokens)?;
//! # Ok(())
//! # }
//! ```
//!
//!
//! FOR PARSER DEVELOPERS
//!
//! When writing tests for parser components, use the TxxtCorpora utility for
//! specification-driven testing. This ensures your implementation matches the
//! authoritative specification.
//!
//! ```ignore
//! use tests::corpora::{TxxtCorpora, ProcessingStage};
//!
//! let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")?;
//! let ast = your_parser::parse(&corpus.source_text)?;
//! insta::assert_yaml_snapshot!(ast);
//! ```
//!
//! See tests/README.md for complete documentation and CLAUDE.md for requirements.
//!
//!
//! DESIGN PRINCIPLES
//!
//! - No backwards compatibility burden (unreleased software)
//! - Specification-driven testing via TxxtCorpora
//! - Clear separation between phases and steps
//! - Flat module structure (no nested pipeline directories)
//! - Pure functions in API layer (no I/O or side effects)
//! - Progressive complexity (simple documents parse easily)
//!
//!
//! ============================================================================
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod annotation_parser;
pub mod api;
pub mod assembler;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod process;
pub mod processing_stages;
pub mod tools;

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub path: PathBuf,
    pub text: String,
    pub source_file: PathBuf,
}

pub type AnnotationMap = HashMap<PathBuf, Annotation>;

pub struct Txxt {
    pub path: PathBuf,
    pub annotations: Vec<Annotation>,
}

impl Txxt {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            annotations: Vec::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref().to_path_buf();
        let mut info_file = Self::new(path.clone());

        if path.exists() {
            info_file.annotations = annotation_parser::parse_file(&path)?;
        }

        Ok(info_file)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        annotation_parser::write_file(&self.path, &self.annotations)
    }
}

pub fn collect_annotations<P: AsRef<Path>>(
    root: P,
) -> Result<AnnotationMap, Box<dyn std::error::Error>> {
    let mut merged: AnnotationMap = HashMap::new();

    for entry in walkdir::WalkDir::new(root.as_ref()) {
        let entry = entry?;
        if entry.file_name() == ".info" {
            let info_file = Txxt::load(entry.path())?;

            for annotation in info_file.annotations {
                let key = annotation.path.clone();

                match merged.get(&key) {
                    Some(existing) => {
                        if is_closer_to_target(&annotation.source_file, &existing.source_file, &key)
                        {
                            merged.insert(key, annotation);
                        }
                    }
                    None => {
                        merged.insert(key, annotation);
                    }
                }
            }
        }
    }

    Ok(merged)
}

fn is_closer_to_target(candidate: &Path, existing: &Path, target: &Path) -> bool {
    let candidate_distance = path_distance(candidate, target);
    let existing_distance = path_distance(existing, target);

    match candidate_distance.cmp(&existing_distance) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => candidate < existing,
    }
}

fn path_distance(from: &Path, to: &Path) -> usize {
    let from_parent = from.parent().unwrap_or(from);
    let to_parent = to.parent().unwrap_or(to);

    let from_components: Vec<_> = from_parent.components().collect();
    let to_components: Vec<_> = to_parent.components().collect();

    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    (from_components.len() - common_len) + (to_components.len() - common_len)
}
// test change
