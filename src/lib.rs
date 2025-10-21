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
//!     · Examples: scanner-tokens, high-level-tokens, ast-block, ast-full
//!     · Stages map to specific steps within phases
//!
//!
//! DATA FLOW
//!
//! The processing pipeline transforms data through these representations:
//! Lexer: source text -> scanner-tokens -> high-level-tokens
//! Parser: -> ast-block -> ast-inlines
//! Assembler:  ast-document -> ast-full
//!
//! PHASE 1: LEXER Converts source text into hierarchical token structures.
//! 1. Lexer: Syntax Analysis: Convert source text into token vectors.
//!     1.a: Step:  Verbatim scanning: Identify and mark verbatim regions
//!         (Raw source text -> Text with verbatim boundaries marked)
//!     1.b: scanner-tokens
//!          Convert text to low-level flat token stream
//!         (Source text with verbatim markers -> Vec<ScannerToken>)
//!     1.c: high-level-tokens
//!          Convert from low-level tokens to high-level tokens
//!          Vec<ScannerToken> -> Vec<HighLevelToken>
//!2. Parser: Semantic Analysis: Converts tokens into Abstract Syntax Tree nodes.
//!    2.a:  ast-block
//!           Build AST tree with block elements from high-level tokens
//!           Purpose: Build AST tree from high-level tokens
//!           Input: HighLevelTokenList
//!           Output: Vec<ElementNode>
//!           Produces: Block-level AST structure
//!     2.b:  ast-inlines
//!           Inline parsing:  Complement tree with inline elements.
//!           Vec<ElementNode> (no inlines) -> Vec<ElementNode> (with  inlines)
//! 3. Assembly: Converts AST nodes into final document structure.
//!     3.a: ast-document
//!          Document assembly: Wrap AST elements in document structure
//!          Vec<ElementNode> -> Document nod, with annotations as core nodes
//!     3.b: ast-full
//!          Annotation attachment : Moves annotations from content to annotation filed.
//!          Document with in content annotations -> Document with annotations in annotation fields
//!
//! DESIGN PRINCIPLES
//! - No backwards compatibility burden (unreleased software)
//! - Specification-driven testing via TxxtCorpora
//! - Clear separation between phases and steps
//! - Pure functions in API layer (no I/O or side effects)
//!
//! ============================================================================
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod annotation_parser;
pub mod api;
pub mod assembler;
pub mod ast;
pub mod cst;
pub mod lexer;
pub mod parser;
pub mod tools;
pub mod transform;

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
