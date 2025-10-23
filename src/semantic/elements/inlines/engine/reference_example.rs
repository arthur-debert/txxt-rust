//! Reference Inline Engine Example
//!
//! Demonstrates the generic inline engine with reference parsing as a complete
//! example. Shows how to:
//! - Create classification transform stages
//! - Build type-based dispatch pipelines
//! - Implement specialized processors for each reference type
//! - Register with the engine

use super::pipeline_data::{ClassifiedSpan, MatchedSpan, StageData, StageError};
use super::{DelimiterSpec, InlineDefinition, Pipeline, PipelineBuilder, Stage};
use crate::ast::elements::formatting::inlines::Inline;
use crate::ast::elements::references::reference_types::{
    CitationEntry, ReferenceClassifier, ReferenceTarget, SimpleReferenceType,
};
use crate::ast::elements::references::Reference;
use crate::cst::{ScannerToken, ScannerTokenSequence};
use std::collections::HashMap;

/// Extract text content from tokens
fn extract_content(tokens: &[ScannerToken]) -> String {
    tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Stage 1: Classify reference type
///
/// Takes a MatchedSpan and determines what type of reference it is
/// (Citation, Footnote, Section, URL, File, TK, NotSure)
fn classify_reference(data: StageData) -> Result<StageData, StageError> {
    let span = data.downcast::<MatchedSpan>()?;

    // Extract content from inner tokens
    let content = extract_content(&span.inner_tokens);

    if content.trim().is_empty() {
        return Err(StageError::InvalidStructure(
            "Reference content cannot be empty".to_string(),
        ));
    }

    // Use ReferenceClassifier to determine type
    let classifier = ReferenceClassifier::new();
    let ref_type = classifier.classify(&content);

    // Map to type name for dispatch
    let type_name = match ref_type {
        SimpleReferenceType::Url => "Url",
        SimpleReferenceType::File => "File",
        SimpleReferenceType::Citation => "Citation",
        SimpleReferenceType::Footnote => "Footnote",
        SimpleReferenceType::Section => "Section",
        SimpleReferenceType::ToComeTK => "ToComeTK",
        SimpleReferenceType::NotSure => "NotSure",
    };

    let classified = ClassifiedSpan {
        type_name: type_name.to_string(),
        span: span.clone(),
    };

    Ok(StageData::new(classified))
}

/// Extract type key from classified span for dispatch
fn extract_reference_type(data: &StageData) -> String {
    if let Ok(classified) = data.downcast::<ClassifiedSpan>() {
        classified.type_name.clone()
    } else {
        "NotSure".to_string()
    }
}

// ============================================================================
// Reference Processors - Convert ClassifiedSpan to Inline
// ============================================================================

/// Process citation reference: [@key1; @key2, locator]
fn process_citation(data: StageData) -> Result<StageData, StageError> {
    let classified = data.downcast::<ClassifiedSpan>()?;
    let content = extract_content(&classified.span.inner_tokens);

    // Parse citation entries
    let entries = parse_citation_entries(&content)?;

    // Build ReferenceTarget
    let target = ReferenceTarget::Citation {
        citations: entries,
        raw: format!("[{}]", content),
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    // Build Reference
    let reference = Reference {
        target,
        content: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    Ok(StageData::new(Inline::Reference(reference)))
}

/// Parse citation entries from content like "@key1; @key2, p. 123"
fn parse_citation_entries(content: &str) -> Result<Vec<CitationEntry>, StageError> {
    let mut entries = Vec::new();

    // Split by semicolon for multiple citations
    for part in content.split(';') {
        let part = part.trim();
        if !part.starts_with('@') {
            continue;
        }

        // Remove @ prefix
        let without_at = &part[1..];

        // Split by comma for locator
        let parts: Vec<&str> = without_at.splitn(2, ',').collect();
        let key = parts[0].trim().to_string();
        let locator = if parts.len() > 1 {
            Some(parts[1].trim().to_string())
        } else {
            None
        };

        entries.push(CitationEntry {
            key,
            locator,
            prefix: None,
            suffix: None,
        });
    }

    if entries.is_empty() {
        return Err(StageError::InvalidStructure(
            "No valid citation entries found".to_string(),
        ));
    }

    Ok(entries)
}

/// Process footnote reference: [1] or [^label]
fn process_footnote(data: StageData) -> Result<StageData, StageError> {
    let classified = data.downcast::<ClassifiedSpan>()?;
    let content = extract_content(&classified.span.inner_tokens);

    // Determine footnote type
    let target = if let Some(label_str) = content.strip_prefix('^') {
        // Named anchor footnote: [^label]
        let label = label_str.to_string();
        ReferenceTarget::NamedAnchor {
            anchor: if label.is_empty() {
                "footnote".to_string()
            } else {
                label
            },
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence {
                tokens: classified.span.full_tokens.clone(),
            },
        }
    } else {
        // Naked numerical footnote: [1]
        let number = content.parse::<u32>().map_err(|_| {
            StageError::InvalidStructure(format!("Invalid footnote number: {}", content))
        })?;
        ReferenceTarget::NakedNumerical {
            number,
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence {
                tokens: classified.span.full_tokens.clone(),
            },
        }
    };

    let reference = Reference {
        target,
        content: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    Ok(StageData::new(Inline::Reference(reference)))
}

/// Process section reference: [#3] or [#2.1.3]
fn process_section(data: StageData) -> Result<StageData, StageError> {
    use crate::ast::elements::references::reference_types::SectionIdentifier;

    let classified = data.downcast::<ClassifiedSpan>()?;
    let content = extract_content(&classified.span.inner_tokens);

    // Parse section identifier
    let identifier = if let Some(number_str) = content.strip_prefix('#') {
        // Numeric section: #3, #2.1, etc.
        if let Some(num_str) = number_str.strip_prefix('-') {
            // Negative indexing: #-1
            let abs_num = num_str.parse::<u32>().unwrap_or(1);
            SectionIdentifier::Numeric {
                levels: vec![abs_num],
                negative_index: true,
            }
        } else {
            // Regular numeric: #3 or #2.1.3
            let levels: Vec<u32> = number_str
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            SectionIdentifier::Numeric {
                levels,
                negative_index: false,
            }
        }
    } else {
        // Named section
        SectionIdentifier::Named {
            name: content.clone(),
        }
    };

    let target = ReferenceTarget::Section {
        identifier,
        raw: format!("[{}]", content),
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    let reference = Reference {
        target,
        content: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    Ok(StageData::new(Inline::Reference(reference)))
}

/// Process URL reference: [https://example.com] or [example.com#section]
fn process_url(data: StageData) -> Result<StageData, StageError> {
    let classified = data.downcast::<ClassifiedSpan>()?;
    let content = extract_content(&classified.span.inner_tokens);

    // Split URL and fragment
    let (url, fragment) = if let Some(pos) = content.find('#') {
        (
            content[..pos].to_string(),
            Some(content[pos + 1..].to_string()),
        )
    } else {
        (content.clone(), None)
    };

    let target = ReferenceTarget::Url {
        url,
        fragment,
        raw: format!("[{}]", content),
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    let reference = Reference {
        target,
        content: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    Ok(StageData::new(Inline::Reference(reference)))
}

/// Process file reference: [./file.txxt] or [../dir/file.txxt#section]
fn process_file(data: StageData) -> Result<StageData, StageError> {
    let classified = data.downcast::<ClassifiedSpan>()?;
    let content = extract_content(&classified.span.inner_tokens);

    // Split path and section
    let (path, section) = if let Some(pos) = content.find('#') {
        (
            content[..pos].to_string(),
            Some(content[pos + 1..].to_string()),
        )
    } else {
        (content.clone(), None)
    };

    let target = ReferenceTarget::File {
        path,
        section,
        raw: format!("[{}]", content),
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    let reference = Reference {
        target,
        content: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    Ok(StageData::new(Inline::Reference(reference)))
}

/// Process TK (to come) placeholder: [TK]
fn process_tk(data: StageData) -> Result<StageData, StageError> {
    let classified = data.downcast::<ClassifiedSpan>()?;
    let content = extract_content(&classified.span.inner_tokens);

    // TK references map to Unresolved with TK prefix
    let target = ReferenceTarget::Unresolved {
        content: content.clone(),
        raw: format!("[{}]", content),
        reason: Some("TK placeholder".to_string()),
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    let reference = Reference {
        target,
        content: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    Ok(StageData::new(Inline::Reference(reference)))
}

/// Process unresolved reference: [unknown]
fn process_not_sure(data: StageData) -> Result<StageData, StageError> {
    let classified = data.downcast::<ClassifiedSpan>()?;
    let content = extract_content(&classified.span.inner_tokens);

    let target = ReferenceTarget::Unresolved {
        content: content.clone(),
        raw: format!("[{}]", content),
        reason: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    let reference = Reference {
        target,
        content: None,
        tokens: ScannerTokenSequence {
            tokens: classified.span.full_tokens.clone(),
        },
    };

    Ok(StageData::new(Inline::Reference(reference)))
}

// ============================================================================
// Pipeline Construction
// ============================================================================

/// Build the complete reference processing pipeline
///
/// Pipeline structure:
/// 1. classify_reference: MatchedSpan → ClassifiedSpan
/// 2. dispatch by type:
///    - Citation → process_citation
///    - Footnote → process_footnote
///    - Section → process_section
///    - Url → process_url
///    - File → process_file
///    - ToComeTK → process_tk
///    - NotSure (default) → process_not_sure
fn build_reference_pipeline() -> Pipeline {
    let mut branches = HashMap::new();

    // Each branch is a pipeline with processing stages
    branches.insert(
        "Citation".to_string(),
        Pipeline::from_stages(vec![Stage::Transform {
            name: "process_citation",
            func: process_citation,
        }]),
    );

    branches.insert(
        "Footnote".to_string(),
        Pipeline::from_stages(vec![Stage::Transform {
            name: "process_footnote",
            func: process_footnote,
        }]),
    );

    branches.insert(
        "Section".to_string(),
        Pipeline::from_stages(vec![Stage::Transform {
            name: "process_section",
            func: process_section,
        }]),
    );

    branches.insert(
        "Url".to_string(),
        Pipeline::from_stages(vec![Stage::Transform {
            name: "process_url",
            func: process_url,
        }]),
    );

    branches.insert(
        "File".to_string(),
        Pipeline::from_stages(vec![Stage::Transform {
            name: "process_file",
            func: process_file,
        }]),
    );

    branches.insert(
        "ToComeTK".to_string(),
        Pipeline::from_stages(vec![Stage::Transform {
            name: "process_tk",
            func: process_tk,
        }]),
    );

    // Default branch for NotSure
    let default = Pipeline::from_stages(vec![Stage::Transform {
        name: "process_not_sure",
        func: process_not_sure,
    }]);

    // Build complete pipeline
    PipelineBuilder::new()
        .then("classify_reference", classify_reference)
        .dispatch("process_by_type", extract_reference_type, branches, default)
        .build()
}

/// Create the reference inline definition for registration
pub fn create_reference_inline() -> InlineDefinition {
    InlineDefinition {
        name: "reference",
        delimiters: DelimiterSpec::new('[', ']'),
        pipeline: build_reference_pipeline(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    fn create_text_token(content: &str) -> ScannerToken {
        ScannerToken::Text {
            content: content.to_string(),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position {
                    row: 0,
                    column: content.len(),
                },
            },
        }
    }

    #[test]
    fn test_classify_reference_citation() {
        let span = MatchedSpan {
            inner_tokens: vec![create_text_token("@smith2023")],
            full_tokens: vec![
                create_text_token("["),
                create_text_token("@smith2023"),
                create_text_token("]"),
            ],
            start: 0,
            end: 3,
            inline_name: "reference".to_string(),
        };

        let result = classify_reference(StageData::new(span));
        assert!(result.is_ok());

        let stage_data = result.unwrap();
        let classified = stage_data.downcast::<ClassifiedSpan>().unwrap();
        assert_eq!(classified.type_name, "Citation");
    }

    #[test]
    fn test_classify_reference_url() {
        let span = MatchedSpan {
            inner_tokens: vec![create_text_token("https://example.com")],
            full_tokens: vec![
                create_text_token("["),
                create_text_token("https://example.com"),
                create_text_token("]"),
            ],
            start: 0,
            end: 3,
            inline_name: "reference".to_string(),
        };

        let result = classify_reference(StageData::new(span));
        assert!(result.is_ok());

        let stage_data = result.unwrap();
        let classified = stage_data.downcast::<ClassifiedSpan>().unwrap();
        assert_eq!(classified.type_name, "Url");
    }

    #[test]
    fn test_process_citation_simple() {
        let span = MatchedSpan {
            inner_tokens: vec![create_text_token("@smith2023")],
            full_tokens: vec![
                create_text_token("["),
                create_text_token("@smith2023"),
                create_text_token("]"),
            ],
            start: 0,
            end: 3,
            inline_name: "reference".to_string(),
        };

        let classified = ClassifiedSpan {
            type_name: "Citation".to_string(),
            span,
        };

        let result = process_citation(StageData::new(classified));
        assert!(result.is_ok());

        let stage_data = result.unwrap();
        let inline = stage_data.downcast::<Inline>().unwrap();
        match inline {
            Inline::Reference(ref_) => match &ref_.target {
                ReferenceTarget::Citation { citations, .. } => {
                    assert_eq!(citations.len(), 1);
                    assert_eq!(citations[0].key, "smith2023");
                    assert!(citations[0].locator.is_none());
                }
                _ => panic!("Expected Citation target"),
            },
            _ => panic!("Expected Reference inline"),
        }
    }

    #[test]
    fn test_parse_citation_entries_multiple() {
        let content = "@smith2023; @jones2025, p. 42";
        let entries = parse_citation_entries(content).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "smith2023");
        assert!(entries[0].locator.is_none());
        assert_eq!(entries[1].key, "jones2025");
        assert_eq!(entries[1].locator.as_ref().unwrap(), "p. 42");
    }

    #[test]
    fn test_reference_pipeline_end_to_end() {
        // Test that we can create the definition without panicking
        let definition = create_reference_inline();
        assert_eq!(definition.name, "reference");
        assert_eq!(definition.delimiters.start, '[');
        assert_eq!(definition.delimiters.end, ']');
        assert!(!definition.pipeline.is_empty());
    }
}
