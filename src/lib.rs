#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]

//! TXXT Parser and Processor
//!
//! # For Parser Developers
//!
//! When writing tests for parser components, use the `TxxtCorpora` utility for
//! specification-driven testing. This ensures your implementation matches the
//! authoritative specification.
//!
//! ```rust,ignore
//! // In your test files:
//! use tests::corpora::{TxxtCorpora, ProcessingStage};
//!
//! let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")?;
//! let ast = your_parser::parse(&corpus.source_text)?;
//! insta::assert_yaml_snapshot!(ast);
//! ```
//!
//! See `tests/README.md` for complete documentation and `CLAUDE.md` for requirements.
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
