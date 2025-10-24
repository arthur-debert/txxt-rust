//! Inline Type Registration
//!
//! This module provides factory functions for creating fully configured
//! InlineEngine instances with all standard inline types registered.
//!
//! # Standard Inline Types
//!
//! - **References**: Citations, footnotes, sections, URLs, files
//!
//! # Usage
//!
//! ```ignore
//! use crate::semantic::elements::inlines::engine::create_standard_engine;
//!
//! let engine = create_standard_engine();
//! let inlines = engine.parse(&tokens);
//! ```

use super::reference_example::build_reference_pipeline;
use super::{DelimiterSpec, InlineDefinition, InlineEngine};

/// Create a fully configured InlineEngine with all standard inline types
///
/// This creates an engine with the following inline types registered:
/// - References: `[...]` with type-based dispatch for citations, footnotes, etc.
///
/// # Errors
///
/// Returns error if any registration fails (e.g., duplicate delimiters)
pub fn create_standard_engine() -> Result<InlineEngine, super::EngineError> {
    let mut engine = InlineEngine::new();

    // Register reference inline type
    engine.register(create_reference_definition())?;

    Ok(engine)
}

/// Create the definition for reference inline type
///
/// References use `[...]` delimiters and include:
/// - Citations: `[@key]`
/// - Footnotes: `[1]`, `[^label]`
/// - Sections: `[#3]`, `[#2.1]`
/// - URLs: `[https://example.com]`
/// - Files: `[./path/to/file.txt]`
fn create_reference_definition() -> InlineDefinition {
    InlineDefinition {
        name: "reference",
        delimiters: DelimiterSpec::new('[', ']'),
        pipeline: build_reference_pipeline(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, ScannerToken, SourceSpan};

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
    fn test_create_standard_engine() {
        let result = create_standard_engine();
        assert!(result.is_ok());

        let engine = result.unwrap();
        assert_eq!(engine.registered_count(), 1);
    }

    #[test]
    fn test_engine_parses_citations() {
        let engine = create_standard_engine().unwrap();

        let tokens = vec![
            create_text_token("["),
            create_text_token("@smith2023"),
            create_text_token("]"),
        ];

        let result = engine.parse(&tokens);
        assert_eq!(result.len(), 1);

        // Should produce a Reference inline
        match &result[0] {
            crate::ast::elements::formatting::inlines::Inline::Reference(ref_) => {
                match &ref_.target {
                    crate::ast::elements::references::ReferenceTarget::Citation {
                        citations,
                        ..
                    } => {
                        assert_eq!(citations.len(), 1);
                        assert_eq!(citations[0].key, "smith2023");
                    }
                    _ => panic!("Expected Citation, got {:?}", ref_.target),
                }
            }
            _ => panic!("Expected Reference inline"),
        }
    }

    #[test]
    fn test_engine_parses_urls() {
        let engine = create_standard_engine().unwrap();

        let tokens = vec![
            create_text_token("["),
            create_text_token("https://example.com"),
            create_text_token("]"),
        ];

        let result = engine.parse(&tokens);
        assert_eq!(result.len(), 1);

        match &result[0] {
            crate::ast::elements::formatting::inlines::Inline::Reference(ref_) => {
                match &ref_.target {
                    crate::ast::elements::references::ReferenceTarget::Url { url, .. } => {
                        assert_eq!(url, "https://example.com");
                    }
                    _ => panic!("Expected Url, got {:?}", ref_.target),
                }
            }
            _ => panic!("Expected Reference inline"),
        }
    }

    #[test]
    fn test_engine_handles_plain_text() {
        let engine = create_standard_engine().unwrap();

        let tokens = vec![
            create_text_token("plain"),
            create_text_token(" "),
            create_text_token("text"),
        ];

        let result = engine.parse(&tokens);
        assert_eq!(result.len(), 3);

        // All should be plain text
        for inline in result {
            match inline {
                crate::ast::elements::formatting::inlines::Inline::TextLine(_) => {}
                _ => panic!("Expected TextLine inline"),
            }
        }
    }
}
