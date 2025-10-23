//! # Level 3: Inline Processors
//!
//! Implementations of InlineProcessor for building final AST nodes from typed spans.
//! This layer performs deep processing including parsing internal structure,
//! handling recursion for nested inlines, and constructing semantic representations.
//!
//! ## Processors
//!
//! - `BoldProcessor`: Builds Strong text transforms with nested content
//! - `ItalicProcessor`: Builds Emphasis text transforms with nested content
//! - `CodeProcessor`: Builds Code text transforms (no nesting)
//! - `MathProcessor`: Builds Math text transforms (no nesting)
//! - `CitationProcessor`: Parses citation keys and locators
//! - `FootnoteProcessor`: Builds footnote references
//! - `SectionProcessor`: Builds section references
//! - `UrlProcessor`: Builds URL references
//! - `FileProcessor`: Builds file references
//! - `TKProcessor`: Builds TK placeholder references
//! - `NotSureProcessor`: Builds unresolved references

use crate::ast::elements::formatting::inlines::{Inline, Text, TextTransform};
use crate::ast::elements::references::reference_types::*;
use crate::cst::{ScannerToken, ScannerTokenSequence};
use crate::semantic::elements::inlines::pipeline::{InlineProcessor, InlineType, TypedSpan};
use crate::semantic::elements::inlines::InlineParseError;

/// Context for preventing same-type nesting
#[derive(Debug, Clone, Copy, PartialEq)]
enum FormattingContext {
    Strong,
    Emphasis,
}

/// Bold/Strong processor - builds TextTransform::Strong with nested content
pub struct BoldProcessor;

impl InlineProcessor for BoldProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Recursively parse inner content, preventing nested bold
        let nested = parse_with_context(&typed_span.span.inner_tokens, FormattingContext::Strong)?;

        Ok(Inline::TextLine(TextTransform::Strong(nested)))
    }
}

/// Italic/Emphasis processor - builds TextTransform::Emphasis with nested content
pub struct ItalicProcessor;

impl InlineProcessor for ItalicProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Recursively parse inner content, preventing nested italic
        let nested =
            parse_with_context(&typed_span.span.inner_tokens, FormattingContext::Emphasis)?;

        Ok(Inline::TextLine(TextTransform::Emphasis(nested)))
    }
}

/// Code processor - builds TextTransform::Code (no nesting allowed)
pub struct CodeProcessor;

impl InlineProcessor for CodeProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Code content is literal - no further parsing
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .map(|t| t.content())
            .collect::<String>();

        let token_sequence = ScannerTokenSequence {
            tokens: typed_span.span.inner_tokens.clone(),
        };

        Ok(Inline::TextLine(TextTransform::Code(
            Text::simple_with_tokens(&content, token_sequence),
        )))
    }
}

/// Math processor - builds TextTransform::Math (no nesting allowed)
pub struct MathProcessor;

impl InlineProcessor for MathProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Math content is literal - no further parsing
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .map(|t| t.content())
            .collect::<String>();

        let token_sequence = ScannerTokenSequence {
            tokens: typed_span.span.inner_tokens.clone(),
        };

        Ok(Inline::TextLine(TextTransform::Math(
            Text::simple_with_tokens(&content, token_sequence),
        )))
    }
}

/// Citation processor - parses citation keys and locators
pub struct CitationProcessor;

impl CitationProcessor {
    /// Parse citation entries from citation content
    fn parse_entries(&self, content: &str) -> Result<Vec<CitationEntry>, InlineParseError> {
        if !content.starts_with('@') {
            return Err(InlineParseError::InvalidStructure(
                "Citation must start with @ symbol".to_string(),
            ));
        }

        // Split by semicolons for multiple citations
        let parts: Vec<&str> = content.split(';').collect();
        let mut citations = Vec::new();

        for part in parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Parse individual citation
            if let Some(citation) = self.parse_single(trimmed)? {
                citations.push(citation);
            }
        }

        if citations.is_empty() {
            return Err(InlineParseError::InvalidStructure(
                "No valid citations found".to_string(),
            ));
        }

        Ok(citations)
    }

    /// Parse a single citation entry
    fn parse_single(&self, citation_str: &str) -> Result<Option<CitationEntry>, InlineParseError> {
        if !citation_str.starts_with('@') {
            return Ok(None);
        }

        let content = &citation_str[1..]; // Remove @ symbol

        // Split by comma to separate key from locator
        let parts: Vec<&str> = content.splitn(2, ',').collect();
        let key = parts[0].trim().to_string();

        if key.is_empty() {
            return Err(InlineParseError::InvalidStructure(
                "Citation key cannot be empty".to_string(),
            ));
        }

        let locator = if parts.len() > 1 {
            let loc = parts[1].trim();
            if loc.is_empty() {
                None
            } else {
                Some(loc.to_string())
            }
        } else {
            None
        };

        Ok(Some(CitationEntry {
            key,
            locator,
            prefix: None,
            suffix: None,
        }))
    }
}

impl InlineProcessor for CitationProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Extract content
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Parse citation entries
        let citations = self.parse_entries(&content)?;

        // Build ReferenceTarget
        let reference_target = ReferenceTarget::Citation {
            citations,
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        // Build Reference AST node
        let reference = crate::ast::elements::references::Reference {
            target: reference_target,
            content: None,
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        Ok(Inline::Reference(reference))
    }
}

/// Footnote processor - builds footnote references
pub struct FootnoteProcessor;

impl InlineProcessor for FootnoteProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Extract content
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Determine if labeled or naked numerical
        let reference_target = if let Some(stripped) = content.strip_prefix('^') {
            // Labeled footnote [^label]
            ReferenceTarget::NamedAnchor {
                anchor: stripped.to_string(),
                raw: format!("[{}]", content),
                tokens: ScannerTokenSequence {
                    tokens: typed_span.span.full_tokens.clone(),
                },
            }
        } else if content.chars().all(|c| c.is_ascii_digit()) {
            // Naked numerical footnote [1]
            let number = content.parse::<u32>().map_err(|_| {
                InlineParseError::InvalidStructure("Invalid footnote number".to_string())
            })?;
            ReferenceTarget::NakedNumerical {
                number,
                raw: format!("[{}]", content),
                tokens: ScannerTokenSequence {
                    tokens: typed_span.span.full_tokens.clone(),
                },
            }
        } else {
            return Err(InlineParseError::InvalidStructure(
                "Invalid footnote format".to_string(),
            ));
        };

        let reference = crate::ast::elements::references::Reference {
            target: reference_target,
            content: None,
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        Ok(Inline::Reference(reference))
    }
}

/// Section processor - builds section/session references
pub struct SectionProcessor;

impl SectionProcessor {
    /// Parse section identifier from content
    fn parse_identifier(&self, content: &str) -> SectionIdentifier {
        let content = content.trim();

        // Check for negative indexing
        let (negative_index, numeric_content) = if let Some(stripped) = content.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, content)
        };

        // Try to parse as numeric levels (e.g., "1.2.3")
        if let Ok(levels) = self.parse_numeric_levels(numeric_content) {
            if !levels.is_empty() {
                return SectionIdentifier::Numeric {
                    levels,
                    negative_index,
                };
            }
        }

        // Fall back to named identifier
        SectionIdentifier::Named {
            name: content.to_string(),
        }
    }

    /// Parse numeric section levels from string
    fn parse_numeric_levels(&self, content: &str) -> Result<Vec<u32>, InlineParseError> {
        if content.is_empty() {
            return Ok(vec![]);
        }

        let parts: Vec<&str> = content.split('.').collect();
        let mut levels = Vec::new();

        for part in parts {
            if let Ok(level) = part.parse::<u32>() {
                levels.push(level);
            } else {
                return Err(InlineParseError::InvalidStructure(
                    "Invalid numeric section level".to_string(),
                ));
            }
        }

        Ok(levels)
    }
}

impl InlineProcessor for SectionProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Extract content
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let identifier = if let Some(stripped) = content.strip_prefix('#') {
            // Parse numeric section reference: #3, #2.1, #-1.2
            self.parse_identifier(stripped)
        } else {
            // Named section reference: local-section
            SectionIdentifier::Named {
                name: content.clone(),
            }
        };

        let reference_target = ReferenceTarget::Section {
            identifier,
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        let reference = crate::ast::elements::references::Reference {
            target: reference_target,
            content: None,
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        Ok(Inline::Reference(reference))
    }
}

/// URL processor - builds URL references
pub struct UrlProcessor;

impl InlineProcessor for UrlProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Extract content
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Parse URL - could have fragment
        let (url, fragment) = if let Some(hash_pos) = content.find('#') {
            let url_part = content[..hash_pos].to_string();
            let fragment_part = content[hash_pos + 1..].to_string();
            (url_part, Some(fragment_part))
        } else {
            (content.clone(), None)
        };

        let reference_target = ReferenceTarget::Url {
            url,
            fragment,
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        let reference = crate::ast::elements::references::Reference {
            target: reference_target,
            content: None,
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        Ok(Inline::Reference(reference))
    }
}

/// File processor - builds file references
pub struct FileProcessor;

impl InlineProcessor for FileProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Extract content
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Parse file path - could have section anchor
        let (path, section) = if let Some(hash_pos) = content.find('#') {
            let path_part = content[..hash_pos].to_string();
            let section_part = content[hash_pos + 1..].to_string();
            (path_part, Some(section_part))
        } else {
            (content.clone(), None)
        };

        let reference_target = ReferenceTarget::File {
            path,
            section,
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        let reference = crate::ast::elements::references::Reference {
            target: reference_target,
            content: None,
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        Ok(Inline::Reference(reference))
    }
}

/// TK processor - builds TK placeholder references
pub struct TKProcessor;

impl InlineProcessor for TKProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Extract content
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let reference_target = ReferenceTarget::Unresolved {
            content: content.clone(),
            raw: format!("[{}]", content),
            reason: Some("TK placeholder".to_string()),
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        let reference = crate::ast::elements::references::Reference {
            target: reference_target,
            content: None,
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        Ok(Inline::Reference(reference))
    }
}

/// NotSure processor - builds unresolved references
pub struct NotSureProcessor;

impl InlineProcessor for NotSureProcessor {
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError> {
        // Extract content
        let content = typed_span
            .span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let reference_target = ReferenceTarget::Unresolved {
            content: content.clone(),
            raw: format!("[{}]", content),
            reason: Some("Unresolved reference type".to_string()),
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        let reference = crate::ast::elements::references::Reference {
            target: reference_target,
            content: None,
            tokens: ScannerTokenSequence {
                tokens: typed_span.span.full_tokens.clone(),
            },
        };

        Ok(Inline::Reference(reference))
    }
}

/// Parse tokens with formatting context to prevent same-type nesting
fn parse_with_context(
    tokens: &[ScannerToken],
    context: FormattingContext,
) -> Result<Vec<TextTransform>, InlineParseError> {
    // For now, use simple recursive parsing
    // TODO: Integrate with full pipeline to handle all inline types
    parse_formatting_recursive(tokens, context)
}

/// Recursive formatting parser (temporary - will be replaced with full pipeline)
fn parse_formatting_recursive(
    tokens: &[ScannerToken],
    context: FormattingContext,
) -> Result<Vec<TextTransform>, InlineParseError> {
    let mut transforms = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        if token.is_bold_delimiter() && context != FormattingContext::Strong {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_bold_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let nested = parse_formatting_recursive(content_tokens, FormattingContext::Strong)?;
                transforms.push(TextTransform::Strong(nested));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else if token.is_italic_delimiter() && context != FormattingContext::Emphasis {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_italic_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let nested =
                    parse_formatting_recursive(content_tokens, FormattingContext::Emphasis)?;
                transforms.push(TextTransform::Emphasis(nested));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else if token.is_code_delimiter() {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_code_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let text = content_tokens
                    .iter()
                    .map(|t| t.content())
                    .collect::<String>();
                let token_sequence = ScannerTokenSequence {
                    tokens: content_tokens.to_vec(),
                };
                transforms.push(TextTransform::Code(Text::simple_with_tokens(
                    &text,
                    token_sequence,
                )));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else if token.is_math_delimiter() {
            if let Some(j) = find_closing_token(tokens, i + 1, |t| t.is_math_delimiter()) {
                let content_tokens = &tokens[i + 1..j];
                let text = content_tokens
                    .iter()
                    .map(|t| t.content())
                    .collect::<String>();
                let token_sequence = ScannerTokenSequence {
                    tokens: content_tokens.to_vec(),
                };
                transforms.push(TextTransform::Math(Text::simple_with_tokens(
                    &text,
                    token_sequence,
                )));
                i = j + 1;
            } else {
                transforms.push(token_to_identity(token));
                i += 1;
            }
        } else {
            transforms.push(token_to_identity(token));
            i += 1;
        }
    }

    Ok(transforms)
}

fn find_closing_token<P>(tokens: &[ScannerToken], start: usize, predicate: P) -> Option<usize>
where
    P: Fn(&ScannerToken) -> bool,
{
    tokens[start..]
        .iter()
        .position(predicate)
        .map(|pos| start + pos)
}

fn token_to_identity(token: &ScannerToken) -> TextTransform {
    let token_sequence = ScannerTokenSequence {
        tokens: vec![token.clone()],
    };
    TextTransform::Identity(Text::simple_with_tokens(token.content(), token_sequence))
}

/// Get processor for inline type
pub fn get_processor(inline_type: &InlineType) -> Box<dyn InlineProcessor> {
    match inline_type {
        InlineType::Bold => Box::new(BoldProcessor),
        InlineType::Italic => Box::new(ItalicProcessor),
        InlineType::Code => Box::new(CodeProcessor),
        InlineType::Math => Box::new(MathProcessor),
        InlineType::Citation => Box::new(CitationProcessor),
        InlineType::Footnote => Box::new(FootnoteProcessor),
        InlineType::Section => Box::new(SectionProcessor),
        InlineType::Url => Box::new(UrlProcessor),
        InlineType::File => Box::new(FileProcessor),
        InlineType::ToComeTK => Box::new(TKProcessor),
        InlineType::NotSure => Box::new(NotSureProcessor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};
    use crate::semantic::elements::inlines::pipeline::SpanMatch;

    fn create_text(content: &str) -> ScannerToken {
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

    fn create_typed_span(inline_type: InlineType, content_tokens: Vec<ScannerToken>) -> TypedSpan {
        let full_tokens = content_tokens.clone();
        TypedSpan {
            span: SpanMatch {
                start: 0,
                end: content_tokens.len(),
                matcher_name: "test".to_string(),
                inner_tokens: content_tokens,
                full_tokens,
            },
            inline_type,
        }
    }

    // ============================================================================
    // Unit Tests for CitationProcessor (Complex Parsing Logic)
    // ============================================================================

    #[test]
    fn test_citation_processor_simple() {
        let processor = CitationProcessor;
        let typed_span = create_typed_span(InlineType::Citation, vec![create_text("@smith2023")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Citation { citations, .. } => {
                    assert_eq!(citations.len(), 1);
                    assert_eq!(citations[0].key, "smith2023");
                    assert_eq!(citations[0].locator, None);
                }
                _ => panic!("Expected Citation target"),
            }
        } else {
            panic!("Expected Reference inline");
        }
    }

    #[test]
    fn test_citation_processor_with_locator() {
        let processor = CitationProcessor;
        let typed_span = create_typed_span(
            InlineType::Citation,
            vec![create_text("@smith2023, p. 123")],
        );

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Citation { citations, .. } => {
                    assert_eq!(citations.len(), 1);
                    assert_eq!(citations[0].key, "smith2023");
                    assert_eq!(citations[0].locator, Some("p. 123".to_string()));
                }
                _ => panic!("Expected Citation target"),
            }
        }
    }

    #[test]
    fn test_citation_processor_multiple_citations() {
        let processor = CitationProcessor;
        let typed_span = create_typed_span(
            InlineType::Citation,
            vec![create_text("@smith2023; @jones2025")],
        );

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Citation { citations, .. } => {
                    assert_eq!(citations.len(), 2);
                    assert_eq!(citations[0].key, "smith2023");
                    assert_eq!(citations[1].key, "jones2025");
                }
                _ => panic!("Expected Citation target"),
            }
        }
    }

    #[test]
    fn test_citation_processor_complex_with_locators() {
        let processor = CitationProcessor;
        let typed_span = create_typed_span(
            InlineType::Citation,
            vec![create_text(
                "@smith2023, ch. 2, p. 45; @jones2025, sec. 3.1",
            )],
        );

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Citation { citations, .. } => {
                    assert_eq!(citations.len(), 2);
                    assert_eq!(citations[0].key, "smith2023");
                    assert_eq!(citations[0].locator, Some("ch. 2, p. 45".to_string()));
                    assert_eq!(citations[1].key, "jones2025");
                    assert_eq!(citations[1].locator, Some("sec. 3.1".to_string()));
                }
                _ => panic!("Expected Citation target"),
            }
        }
    }

    #[test]
    fn test_citation_processor_empty_key_error() {
        let processor = CitationProcessor;
        let typed_span = create_typed_span(InlineType::Citation, vec![create_text("@")]);

        let result = processor.process(&typed_span);
        assert!(result.is_err());
        assert!(matches!(result, Err(InlineParseError::InvalidStructure(_))));
    }

    #[test]
    fn test_citation_processor_missing_at_sign_error() {
        let processor = CitationProcessor;
        let typed_span = create_typed_span(InlineType::Citation, vec![create_text("smith2023")]);

        let result = processor.process(&typed_span);
        assert!(result.is_err());
    }

    // ============================================================================
    // Unit Tests for FootnoteProcessor
    // ============================================================================

    #[test]
    fn test_footnote_processor_naked_numerical() {
        let processor = FootnoteProcessor;
        let typed_span = create_typed_span(InlineType::Footnote, vec![create_text("1")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::NakedNumerical { number, .. } => {
                    assert_eq!(*number, 1);
                }
                _ => panic!("Expected NakedNumerical target"),
            }
        }
    }

    #[test]
    fn test_footnote_processor_labeled() {
        let processor = FootnoteProcessor;
        let typed_span = create_typed_span(InlineType::Footnote, vec![create_text("^note-label")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::NamedAnchor { anchor, .. } => {
                    assert_eq!(anchor, "note-label");
                }
                _ => panic!("Expected NamedAnchor target"),
            }
        }
    }

    #[test]
    fn test_footnote_processor_invalid_format_error() {
        let processor = FootnoteProcessor;
        let typed_span = create_typed_span(InlineType::Footnote, vec![create_text("not-a-number")]);

        let result = processor.process(&typed_span);
        assert!(result.is_err());
    }

    // ============================================================================
    // Unit Tests for SectionProcessor (Identifier Parsing)
    // ============================================================================

    #[test]
    fn test_section_processor_simple_numeric() {
        let processor = SectionProcessor;
        let typed_span = create_typed_span(InlineType::Section, vec![create_text("#3")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Section { identifier, .. } => match identifier {
                    SectionIdentifier::Numeric {
                        levels,
                        negative_index,
                    } => {
                        assert_eq!(levels, &vec![3]);
                        assert!(!negative_index);
                    }
                    _ => panic!("Expected Numeric identifier"),
                },
                _ => panic!("Expected Section target"),
            }
        }
    }

    #[test]
    fn test_section_processor_hierarchical() {
        let processor = SectionProcessor;
        let typed_span = create_typed_span(InlineType::Section, vec![create_text("#2.1.3")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Section { identifier, .. } => match identifier {
                    SectionIdentifier::Numeric { levels, .. } => {
                        assert_eq!(levels, &vec![2, 1, 3]);
                    }
                    _ => panic!("Expected Numeric identifier"),
                },
                _ => panic!("Expected Section target"),
            }
        }
    }

    #[test]
    fn test_section_processor_negative_indexing() {
        let processor = SectionProcessor;
        let typed_span = create_typed_span(InlineType::Section, vec![create_text("#-1.2")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Section { identifier, .. } => match identifier {
                    SectionIdentifier::Numeric {
                        levels,
                        negative_index,
                    } => {
                        assert_eq!(levels, &vec![1, 2]);
                        assert!(negative_index);
                    }
                    _ => panic!("Expected Numeric identifier"),
                },
                _ => panic!("Expected Section target"),
            }
        }
    }

    #[test]
    fn test_section_processor_named() {
        let processor = SectionProcessor;
        let typed_span = create_typed_span(InlineType::Section, vec![create_text("introduction")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Section { identifier, .. } => match identifier {
                    SectionIdentifier::Named { name } => {
                        assert_eq!(name, "introduction");
                    }
                    _ => panic!("Expected Named identifier"),
                },
                _ => panic!("Expected Section target"),
            }
        }
    }

    // ============================================================================
    // Unit Tests for URL and File Processors (Fragment Parsing)
    // ============================================================================

    #[test]
    fn test_url_processor_simple() {
        let processor = UrlProcessor;
        let typed_span =
            create_typed_span(InlineType::Url, vec![create_text("https://example.com")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Url {
                    url, fragment, raw, ..
                } => {
                    assert_eq!(url, "https://example.com");
                    assert_eq!(fragment, &None);
                    assert_eq!(raw, "[https://example.com]");
                }
                _ => panic!("Expected Url target"),
            }
        }
    }

    #[test]
    fn test_url_processor_with_fragment() {
        let processor = UrlProcessor;
        let typed_span = create_typed_span(
            InlineType::Url,
            vec![create_text("https://example.com#section")],
        );

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::Url { url, fragment, .. } => {
                    assert_eq!(url, "https://example.com");
                    assert_eq!(fragment, &Some("section".to_string()));
                }
                _ => panic!("Expected Url target"),
            }
        }
    }

    #[test]
    fn test_file_processor_with_section() {
        let processor = FileProcessor;
        let typed_span =
            create_typed_span(InlineType::File, vec![create_text("./file.txt#section")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::Reference(reference)) = result {
            match &reference.target {
                ReferenceTarget::File { path, section, .. } => {
                    assert_eq!(path, "./file.txt");
                    assert_eq!(section, &Some("section".to_string()));
                }
                _ => panic!("Expected File target"),
            }
        }
    }

    // ============================================================================
    // Unit Tests for Code and Math Processors (Literal Content)
    // ============================================================================

    #[test]
    fn test_code_processor_literal_content() {
        let processor = CodeProcessor;
        let typed_span = create_typed_span(InlineType::Code, vec![create_text("let x = 42;")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::TextLine(TextTransform::Code(text))) = result {
            assert_eq!(text.content(), "let x = 42;");
        } else {
            panic!("Expected TextLine(Code)");
        }
    }

    #[test]
    fn test_math_processor_literal_content() {
        let processor = MathProcessor;
        let typed_span = create_typed_span(InlineType::Math, vec![create_text("x^2 + y^2 = z^2")]);

        let result = processor.process(&typed_span);
        assert!(result.is_ok());

        if let Ok(Inline::TextLine(TextTransform::Math(text))) = result {
            assert_eq!(text.content(), "x^2 + y^2 = z^2");
        } else {
            panic!("Expected TextLine(Math)");
        }
    }

    // ============================================================================
    // Unit Tests for get_processor Factory Function
    // ============================================================================

    #[test]
    fn test_get_processor_returns_correct_types() {
        let types = vec![
            InlineType::Bold,
            InlineType::Italic,
            InlineType::Code,
            InlineType::Math,
            InlineType::Citation,
            InlineType::Footnote,
            InlineType::Section,
            InlineType::Url,
            InlineType::File,
            InlineType::ToComeTK,
            InlineType::NotSure,
        ];

        for inline_type in types {
            // Just verify get_processor doesn't panic for each type
            // The actual behavior is tested in individual processor tests
            let _processor = get_processor(&inline_type);
            // If we get here without panicking, the test passes
        }
    }
}
