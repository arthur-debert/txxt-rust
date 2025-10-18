//! # Reference Elements Parser Module
//!
//! This module contains the logic for parsing reference inline elements - citations,
//! footnotes, page references, and session references that provide cross-referencing
//! capabilities within txxt documents.
//!
//! ## Overview
//!
//! Reference elements enable cross-referencing and citation management in txxt documents.
//! They use square bracket tokens `[content]` and follow specific patterns for different
//! types of references. The comprehensive ReferenceTarget system provides complete type
//! information for all reference types.
//!
//! ## Reference Types
//!
//! ### 1. Citations
//! Academic and scholarly references to external sources:
//! - **Simple citations**: `[@smith2023]`, `[@doe2024]`
//! - **Multiple citations**: `[@smith2023; @jones2025]`
//! - **Page-specific citations**: `[@smith2023, p. 123]`
//! - **Complex citations**: `[@smith2023, ch. 2, p. 45; @jones2025, sec. 3.1]`
//!
//! ### 2. Footnotes
//! Inline notes and references:
//! - **Naked footnotes**: `[1]`, `[2]` (shorthand for footnotes)
//! - **Labeled footnotes**: `[^note-label]`, `[^important-note]`
//! - **Auto-generated footnotes**: `[^]` (generates unique ID)
//!
//! ### 3. Page References
//! References to specific pages or page ranges:
//! - **Single pages**: `[page:123]`, `[p. 123]`
//! - **Page ranges**: `[pages:123-125]`, `[pp. 123-125]`
//! - **Chapter references**: `[chapter:5]`, `[ch. 5]`
//! - **Section references**: `[section:3.1]`, `[sec. 3.1]`
//!
//! ### 4. Session References
//! References to document sections and subsections:
//! - **Numbered sessions**: `[#3]`, `[#2.1]`, `[#1.2.3]`
//! - **Named sessions**: `[local-section]`, `[introduction]`
//! - **Hierarchical references**: `[#parent.child]`, `[#chapter.section]`
//! - **Negative indexing**: `[#-1]` (last session), `[#-2]` (second to last)
//!
//! ### 5. General References
//! File references and external links:
//! - **File references**: `[./filename.txxt]`, `[../dir/file.txxt]`
//! - **URL references**: `[https://example.com]`, `[example.com]`
//! - **Named anchor references**: `[#hello-world]` (via ref= parameters)
//!
//! ## Grammar
//!
//! From [`docs/specs/elements/references/references-general.txxt`]:
//!
//! ```text
//! Reference Spans:
//! <reference-span> = <left-bracket> <reference-content> <right-bracket>
//! <citation-span> = <left-bracket> <at-sign> <citation-keys> <citation-locator>? <right-bracket>
//! <page-ref> = <left-bracket> <page-locator> <right-bracket>
//! <session-ref> = <left-bracket> <hash> <session-number> <right-bracket>
//! <footnote-ref> = <footnote-naked> | <footnote-labeled>
//! ```
//!
//! ## AST Structure
//!
//! Post-parsing semantic representation:
//!
//! ```text
//! Reference AST:
//!     ├── Reference
//!     │   ├── target: ReferenceTarget
//!     │   │   ├── CitationTarget
//!     │   │   │   ├── keys: Vec<String>
//!     │   │   │   └── locator: Option<String>
//!     │   │   ├── FootnoteTarget
//!     │   │   │   ├── label: Option<String>
//!     │   │   │   └── auto_generate: bool
//!     │   │   ├── PageTarget
//!     │   │   │   ├── page: Option<u32>
//!     │   │   │   ├── page_range: Option<(u32, u32)>
//!     │   │   │   └── chapter: Option<String>
//!     │   │   ├── SessionTarget
//!     │   │   │   ├── number: Option<String>
//!     │   │   │   ├── name: Option<String>
//!     │   │   │   └── negative_index: Option<i32>
//!     │   │   ├── FileTarget
//!     │   │   │   ├── path: String
//!     │   │   │   └── anchor: Option<String>
//!     │   │   └── UrlTarget
//!     │   │       ├── url: String
//!     │   │       └── display_text: Option<String>
//!     │   ├── content: Option<Vec<Inline>>
//!     │   └── tokens: ScannerTokenSequence
//! ```
//!
//! ## Processing Rules
//!
//! ### Recognition Criteria
//! - Square bracket tokens: `[content]`
//! - Content cannot be empty
//! - Content cannot span line breaks
//! - Specific patterns for each reference type
//! - Case-sensitive matching for most patterns
//!
//! ### Citation Processing
//! 1. **Key Extraction**: Parse citation keys (e.g., `smith2023`)
//! 2. **Locator Parsing**: Extract page/chapter/section info
//! 3. **Multiple Citations**: Handle semicolon-separated lists
//! 4. **Validation**: Check key format and locator syntax
//!
//! ### Reference Target Resolution
//! 1. **Pattern Matching**: Identify reference type by content pattern
//! 2. **Target Parsing**: Extract structured information
//! 3. **Validation**: Ensure valid format for each type
//! 4. **AST Construction**: Build ReferenceTarget with full type information
//!
//! ## Integration with Document Systems
//!
//! References integrate with:
//! - **Citation Management**: Bibliography and citation formatting
//! - **Cross-Reference Resolution**: Link validation and navigation
//! - **Page Numbering**: Dynamic page reference updates
//! - **Table of Contents**: Session reference navigation
//! - **Footnotes**: Automatic footnote numbering and placement
//!
//! ## Related Files
//!
//! - **Specification**: [`docs/specs/elements/references/references-general.txxt`]
//! - **Citations Spec**: [`docs/specs/elements/references/citations.txxt`]
//! - **AST Nodes**: [`src/ast/elements/references/`]
//! - **Reference Types**: [`src/ast/elements/references/reference_types.rs`]
//! - **Tokenizer**: [`src/lexer/elements/inline/references/`]
//!
//! ## Testing Strategy
//!
//! 1. **Pattern Recognition**: Test each reference type pattern
//! 2. **Target Parsing**: Validate structured data extraction
//! 3. **Edge Cases**: Empty content, malformed patterns, invalid formats
//! 4. **Integration**: Mixed content with formatting, complex documents
//!
//! ## Implementation Notes
//!
//! - Use comprehensive ReferenceTarget system for type safety
//! - Maintain token-level precision for language server support
//! - Support progressive complexity (simple → multiple → complex)
//! - Handle validation errors gracefully with meaningful messages
//! - Integrate with document-wide reference resolution systems

pub mod citations;
pub mod footnote_ref;
pub mod page_ref;
pub mod session_ref;

// Re-export reference parsing functions
pub use citations::*;
pub use footnote_ref::*;
pub use page_ref::*;
pub use session_ref::*;

// Import necessary types
use crate::ast::elements::formatting::inlines::Inline;
use crate::ast::elements::references::reference_types::*;
use crate::ast::elements::scanner_tokens::ScannerTokenSequence;
use crate::parser::elements::inlines::InlineParseError;

/// Extract reference content from bracketed tokens
///
/// Removes the opening and closing brackets and returns the inner content.
/// This is used by all reference type parsers.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing bracketed reference
///
/// # Returns
/// * `Result<String, InlineParseError>` - Content between brackets
fn extract_reference_content(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<String, InlineParseError> {
    if tokens.len() < 3 {
        return Err(InlineParseError::InvalidStructure(
            "Reference must have at least opening bracket, content, and closing bracket"
                .to_string(),
        ));
    }

    // Check for proper bracket pattern
    let first_token = &tokens[0];
    let last_token = &tokens[tokens.len() - 1];

    let starts_with_bracket =
        matches!(first_token, crate::ast::scanner_tokens::ScannerToken::Text { content, .. } if content == "[");
    let ends_with_bracket =
        matches!(last_token, crate::ast::scanner_tokens::ScannerToken::Text { content, .. } if content == "]");

    if !starts_with_bracket || !ends_with_bracket {
        return Err(InlineParseError::InvalidStructure(
            "Reference must be enclosed in square brackets".to_string(),
        ));
    }

    // Extract content tokens (everything between brackets)
    let content_tokens = &tokens[1..tokens.len() - 1];

    if content_tokens.is_empty() {
        return Err(InlineParseError::EmptyContent(
            "Reference content cannot be empty".to_string(),
        ));
    }

    // Convert tokens to string content
    let content = content_tokens
        .iter()
        .filter_map(|token| match token {
            crate::ast::scanner_tokens::ScannerToken::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    if content.trim().is_empty() {
        return Err(InlineParseError::EmptyContent(
            "Reference content cannot be empty".to_string(),
        ));
    }

    Ok(content)
}

/// Parse citation entries from citation content
///
/// Handles formats like "@key1; @key2, p. 123" and extracts individual citations.
///
/// # Arguments
/// * `content` - Citation content string (without brackets)
///
/// # Returns
/// * `Result<Vec<CitationEntry>, InlineParseError>` - Parsed citation entries
fn parse_citation_entries(content: &str) -> Result<Vec<CitationEntry>, InlineParseError> {
    if !content.starts_with('@') {
        return Err(InlineParseError::InvalidStructure(
            "Citation must start with @ symbol".to_string(),
        ));
    }

    // Simple citation parsing - split by semicolons for multiple citations
    let citation_parts: Vec<&str> = content.split(';').collect();
    let mut citations = Vec::new();

    for part in citation_parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse individual citation: @key or @key, locator
        if let Some(citation) = parse_single_citation(trimmed)? {
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
///
/// Handles formats like "@key" or "@key, p. 123".
///
/// # Arguments
/// * `citation_str` - Single citation string
///
/// # Returns
/// * `Result<Option<CitationEntry>, InlineParseError>` - Parsed citation or None if invalid
fn parse_single_citation(citation_str: &str) -> Result<Option<CitationEntry>, InlineParseError> {
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

/// Parse section identifier from section reference content
///
/// Handles numeric formats like "3", "2.1", "-1.2" and mixed formats.
///
/// # Arguments
/// * `content` - Section reference content (after # symbol)
///
/// # Returns
/// * `SectionIdentifier` - Parsed section identifier
fn parse_section_identifier(content: &str) -> SectionIdentifier {
    let content = content.trim();

    // Check for negative indexing
    let (negative_index, numeric_content) = if let Some(stripped) = content.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, content)
    };

    // Try to parse as numeric levels (e.g., "1.2.3")
    if let Ok(levels) = parse_numeric_levels(numeric_content) {
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
///
/// Converts "1.2.3" to vec![1, 2, 3].
///
/// # Arguments
/// * `content` - Numeric content string
///
/// # Returns
/// * `Result<Vec<u32>, InlineParseError>` - Parsed numeric levels
fn parse_numeric_levels(content: &str) -> Result<Vec<u32>, InlineParseError> {
    if content.is_empty() {
        return Ok(vec![]);
    }

    let parts: Vec<&str> = content.split('.').collect();
    let mut levels = Vec::new();

    for part in parts {
        if let Ok(level) = part.parse::<u32>() {
            levels.push(level);
        } else {
            // If any part isn't numeric, this isn't a numeric identifier
            return Err(InlineParseError::InvalidStructure(
                "Invalid numeric section level".to_string(),
            ));
        }
    }

    Ok(levels)
}

/// General reference parser that dispatches to specific type parsers
///
/// Uses the ReferenceClassifier to determine reference type and route to
/// the appropriate parser function.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing reference
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
pub fn parse_reference(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<
    crate::ast::elements::formatting::inlines::Inline,
    crate::parser::elements::inlines::InlineParseError,
> {
    if tokens.is_empty() {
        return Err(InlineParseError::InvalidStructure(
            "Empty reference tokens".to_string(),
        ));
    }

    // Extract content to determine reference type
    let content = extract_reference_content(tokens)?;

    // Classify reference type using the specification order
    let classifier = ReferenceClassifier::new();
    let ref_type = classifier.classify(&content);

    // Route to appropriate parser based on type
    match ref_type {
        SimpleReferenceType::Citation => parse_citation(tokens),
        SimpleReferenceType::Footnote => parse_footnote_ref(tokens),
        SimpleReferenceType::Section => parse_session_ref(tokens),
        SimpleReferenceType::Url => parse_url_reference(tokens),
        SimpleReferenceType::File => parse_file_reference(tokens),
        SimpleReferenceType::ToComeTK => parse_tk_reference(tokens),
        SimpleReferenceType::NotSure => parse_not_sure_reference(tokens),
    }
}

/// Parse URL reference from tokens
///
/// Handles URL patterns like "https://example.com" or "example.com".
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing URL reference
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
fn parse_url_reference(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError> {
    let content = extract_reference_content(tokens)?;

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
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}

/// Parse file reference from tokens
///
/// Handles file patterns like "./file.txt" or "../dir/file.txt".
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing file reference
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
fn parse_file_reference(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError> {
    let content = extract_reference_content(tokens)?;

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
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}

/// Parse TK (To Come) reference from tokens
///
/// Handles TK patterns like "TK" or "TK-identifier".
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing TK reference
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
fn parse_tk_reference(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError> {
    let content = extract_reference_content(tokens)?;

    // TK references are treated as unresolved placeholders
    let reference_target = ReferenceTarget::Unresolved {
        content: content.clone(),
        raw: format!("[{}]", content),
        reason: Some("TK placeholder".to_string()),
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}

/// Parse unresolved/not-sure reference from tokens
///
/// Handles references that don't match any specific pattern.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing unresolved reference
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
fn parse_not_sure_reference(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError> {
    let content = extract_reference_content(tokens)?;

    let reference_target = ReferenceTarget::Unresolved {
        content: content.clone(),
        raw: format!("[{}]", content),
        reason: Some("Unresolved reference type".to_string()),
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}

/// Parse citation references from tokens
///
/// Handles academic and scholarly citations using the `[@key]` pattern.
/// Supports multiple citations, page locators, and complex citation formats.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing citation content
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
///
/// # Examples
///
/// ```rust,ignore
/// // Simple citation
/// parse_citation(&tokens_for("[@smith2023]"))?;
///
/// // Multiple citations
/// parse_citation(&tokens_for("[@smith2023; @jones2025]"))?;
///
/// // Page-specific citation
/// parse_citation(&tokens_for("[@smith2023, p. 123]"))?;
/// ```
pub fn parse_citation(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<
    crate::ast::elements::formatting::inlines::Inline,
    crate::parser::elements::inlines::InlineParseError,
> {
    use crate::ast::elements::references::reference_types::*;
    use crate::ast::elements::scanner_tokens::ScannerTokenSequence;

    if tokens.is_empty() {
        return Err(
            crate::parser::elements::inlines::InlineParseError::InvalidStructure(
                "Empty citation tokens".to_string(),
            ),
        );
    }

    // Extract content from bracket pattern
    let content = extract_reference_content(tokens)?;

    // Parse citation pattern (@key1; @key2, p. 123)
    let citations = parse_citation_entries(&content)?;

    if citations.is_empty() {
        return Err(
            crate::parser::elements::inlines::InlineParseError::InvalidStructure(
                "No valid citation entries found".to_string(),
            ),
        );
    }

    // Create Reference AST node with Citation target
    let reference_target = ReferenceTarget::Citation {
        citations,
        raw: format!("[{}]", content),
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}

/// Parse footnote references from tokens
///
/// Handles footnote references using naked numbers `[1]` or labeled format `[^label]`.
/// Supports auto-generated footnotes and custom labels.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing footnote content
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
///
/// # Examples
///
/// ```rust,ignore
/// // Naked footnote
/// parse_footnote_ref(&tokens_for("[1]"))?;
///
/// // Labeled footnote
/// parse_footnote_ref(&tokens_for("[^note-label]"))?;
///
/// // Auto-generated footnote
/// parse_footnote_ref(&tokens_for("[^]"))?;
/// ```
pub fn parse_footnote_ref(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<
    crate::ast::elements::formatting::inlines::Inline,
    crate::parser::elements::inlines::InlineParseError,
> {
    use crate::ast::elements::references::reference_types::*;
    use crate::ast::elements::scanner_tokens::ScannerTokenSequence;

    if tokens.is_empty() {
        return Err(
            crate::parser::elements::inlines::InlineParseError::InvalidStructure(
                "Empty footnote tokens".to_string(),
            ),
        );
    }

    // Extract content from bracket pattern
    let content = extract_reference_content(tokens)?;

    // Determine if this is naked numerical or labeled footnote
    let reference_target = if let Some(stripped) = content.strip_prefix('^') {
        // Labeled footnote [^label]
        let label = stripped.to_string();
        ReferenceTarget::NamedAnchor {
            anchor: label,
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
        }
    } else if content.chars().all(|c| c.is_ascii_digit()) {
        // Naked numerical footnote [1]
        let number = content.parse::<u32>().map_err(|_| {
            crate::parser::elements::inlines::InlineParseError::InvalidStructure(
                "Invalid footnote number".to_string(),
            )
        })?;
        ReferenceTarget::NakedNumerical {
            number,
            raw: format!("[{}]", content),
            tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
        }
    } else {
        return Err(
            crate::parser::elements::inlines::InlineParseError::InvalidStructure(
                "Invalid footnote format".to_string(),
            ),
        );
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}

/// Parse page references from tokens
///
/// Handles page references using patterns like `[page:123]`, `[pages:123-125]`,
/// `[chapter:5]`, or `[section:3.1]`.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing page reference content
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
///
/// # Examples
///
/// ```rust,ignore
/// // Single page
/// parse_page_ref(&tokens_for("[page:123]"))?;
///
/// // Page range
/// parse_page_ref(&tokens_for("[pages:123-125]"))?;
///
/// // Chapter reference
/// parse_page_ref(&tokens_for("[chapter:5]"))?;
/// ```
pub fn parse_page_ref(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<
    crate::ast::elements::formatting::inlines::Inline,
    crate::parser::elements::inlines::InlineParseError,
> {
    if tokens.is_empty() {
        return Err(
            crate::parser::elements::inlines::InlineParseError::InvalidStructure(
                "Empty page reference tokens".to_string(),
            ),
        );
    }

    // Extract content from bracket pattern
    let content = extract_reference_content(tokens)?;

    // Page references are treated as unresolved for now
    // In the future, this could parse page:123, pages:123-125, etc.
    let reference_target = ReferenceTarget::Unresolved {
        content: content.clone(),
        raw: format!("[{}]", content),
        reason: Some("Page reference not fully implemented".to_string()),
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}

/// Parse session references from tokens
///
/// Handles session references using numbered format `[#3]`, `[#2.1]` or named format
/// `[local-section]`. Supports hierarchical references and negative indexing.
///
/// # Arguments
/// * `tokens` - Sequence of tokens containing session reference content
///
/// # Returns
/// * `Result<crate::ast::elements::formatting::inlines::Inline, InlineParseError>`
///
/// # Examples
///
/// ```rust,ignore
/// // Numbered session
/// parse_session_ref(&tokens_for("[#3]"))?;
///
/// // Hierarchical session
/// parse_session_ref(&tokens_for("[#2.1]"))?;
///
/// // Named session
/// parse_session_ref(&tokens_for("[local-section]"))?;
/// ```
pub fn parse_session_ref(
    tokens: &[crate::ast::scanner_tokens::ScannerToken],
) -> Result<
    crate::ast::elements::formatting::inlines::Inline,
    crate::parser::elements::inlines::InlineParseError,
> {
    use crate::ast::elements::references::reference_types::*;
    use crate::ast::elements::scanner_tokens::ScannerTokenSequence;

    if tokens.is_empty() {
        return Err(
            crate::parser::elements::inlines::InlineParseError::InvalidStructure(
                "Empty session reference tokens".to_string(),
            ),
        );
    }

    // Extract content from bracket pattern
    let content = extract_reference_content(tokens)?;

    let identifier = if let Some(stripped) = content.strip_prefix('#') {
        // Parse numeric section reference: #3, #2.1, #-1.2
        parse_section_identifier(stripped)
    } else {
        // Named section reference: local-section
        SectionIdentifier::Named {
            name: content.clone(),
        }
    };

    let reference_target = ReferenceTarget::Section {
        identifier,
        raw: format!("[{}]", content),
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    let reference = crate::ast::elements::references::Reference {
        target: reference_target,
        content: None,
        tokens: ScannerTokenSequence::from_tokens(tokens.to_vec()),
    };

    Ok(Inline::Reference(reference))
}
