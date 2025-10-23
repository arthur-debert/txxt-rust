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
