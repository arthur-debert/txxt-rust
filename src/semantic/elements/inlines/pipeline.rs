//! # Declarative Multi-Level Inline Parsing Pipeline
//!
//! This module implements a three-level pipeline architecture for parsing inline elements:
//!
//! **Level 1: Delimiter Matching**
//! - Scans token stream for delimiter pairs (*, _, `, #, [])
//! - Identifies span boundaries without understanding content
//! - Enforces structural constraints (single-line, balanced delimiters)
//!
//! **Level 2: Type Classification**
//! - Determines specific inline type from matched spans
//! - For formatting: type implied by delimiter (* = bold, _ = italic, etc.)
//! - For references: uses ReferenceClassifier to determine URL, File, Citation, etc.
//!
//! **Level 3: Deep Processing**
//! - Parses internal structure (citation keys, nested formatting, etc.)
//! - Builds final AST nodes with complete semantic information
//! - Handles recursive processing for nested inlines
//!
//! ## Architecture Benefits
//!
//! - **Declarative**: Inline processors defined as composable traits
//! - **Testable**: Each level can be tested independently
//! - **Extensible**: New inline types added by implementing traits
//! - **Maintainable**: Clear separation of concerns across levels

use crate::ast::elements::formatting::inlines::{Inline, Text, TextTransform};
use crate::cst::{ScannerToken, ScannerTokenSequence};
use crate::semantic::elements::inlines::InlineParseError;

/// Level 1: Matched span with delimiter information
///
/// Represents a span of tokens that has been identified as a potential
/// inline element based on delimiter matching. No semantic interpretation
/// has been performed yet.
#[derive(Debug, Clone)]
pub struct SpanMatch {
    /// Start position in token stream
    pub start: usize,

    /// End position in token stream (exclusive)
    pub end: usize,

    /// Name of the matcher that identified this span
    pub matcher_name: String,

    /// Tokens between delimiters (excluding the delimiters themselves)
    pub inner_tokens: Vec<ScannerToken>,

    /// All tokens including delimiters (for AST token preservation)
    pub full_tokens: Vec<ScannerToken>,
}

/// Level 1: Delimiter Matcher trait
///
/// Implementations scan a token stream and identify spans enclosed by
/// specific delimiters. They do not interpret content or build AST nodes.
///
/// # Examples
///
/// - `BoldMatcher`: Matches `*...*` patterns
/// - `ItalicMatcher`: Matches `_..._` patterns
/// - `ReferenceMatcher`: Matches `[...]` patterns
pub trait DelimiterMatcher {
    /// Name of this matcher (for debugging and dispatch)
    fn name(&self) -> &str;

    /// Attempt to match a span starting at the given position
    ///
    /// Returns Some(SpanMatch) if a valid span is found, None otherwise.
    /// Should enforce structural constraints like single-line content.
    fn match_span(&self, tokens: &[ScannerToken], start: usize) -> Option<SpanMatch>;

    /// Check if this matcher should process the token at this position
    fn can_start(&self, token: &ScannerToken) -> bool;
}

/// Level 2: Typed span with semantic information
///
/// Represents a matched span that has been classified into a specific
/// inline element type. Ready for deep processing in Level 3.
#[derive(Debug, Clone)]
pub struct TypedSpan {
    /// The matched span from Level 1
    pub span: SpanMatch,

    /// The classified inline type
    pub inline_type: InlineType,
}

/// Inline element type classification
///
/// This enum represents all possible inline element types that can be
/// identified during Level 2 processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineType {
    // Formatting elements
    Bold,
    Italic,
    Code,
    Math,

    // Reference elements (classified by ReferenceClassifier)
    Citation,
    Footnote,
    Section,
    Url,
    File,
    ToComeTK,
    NotSure,
}

/// Level 2: Type Classifier trait
///
/// Implementations determine the specific inline element type from a
/// matched span. For formatting, the type is usually implied by the
/// delimiter. For references, pattern analysis is required.
pub trait TypeClassifier {
    /// Classify a matched span into a specific inline type
    fn classify(&self, span: &SpanMatch) -> Result<InlineType, InlineParseError>;
}

/// Level 3: Inline Processor trait
///
/// Implementations perform deep processing on typed spans to build
/// final AST nodes. This includes parsing internal structure,
/// handling recursion, and constructing semantic representations.
pub trait InlineProcessor {
    /// Process a typed span into a final inline AST node
    fn process(&self, typed_span: &TypedSpan) -> Result<Inline, InlineParseError>;
}

/// Multi-level inline parsing pipeline
///
/// Orchestrates the three-level parsing process:
/// 1. Match delimiters to identify potential inline spans
/// 2. Classify matched spans into specific inline types
/// 3. Process typed spans into final AST nodes
pub struct InlinePipeline {
    /// Level 1: Delimiter matchers
    matchers: Vec<Box<dyn DelimiterMatcher>>,
}

impl InlinePipeline {
    /// Create a new pipeline with default matchers
    pub fn new() -> Self {
        Self {
            matchers: Vec::new(),
        }
    }

    /// Add a delimiter matcher to the pipeline
    pub fn with_matcher(mut self, matcher: Box<dyn DelimiterMatcher>) -> Self {
        self.matchers.push(matcher);
        self
    }

    /// Parse a token stream into inline elements
    ///
    /// This is the main entry point that orchestrates all three levels:
    /// 1. Scan for delimiter matches (Level 1)
    /// 2. Classify matched spans (Level 2)
    /// 3. Process into final AST (Level 3)
    pub fn parse(&self, tokens: &[ScannerToken]) -> Result<Vec<Inline>, InlineParseError> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // Level 1: Try to match delimiters
            if let Some((span, matcher_name)) = self.try_match_at(tokens, i) {
                // Level 2: Classify the matched span
                let typed_span = self.classify_span(span, &matcher_name)?;

                // Save end position before moving typed_span
                let next_i = typed_span.span.end;

                // Level 3: Process into final AST
                let inline = self.process_span(typed_span)?;

                result.push(inline);
                i = next_i;
            } else {
                // No match - treat as plain text
                let text_inline = self.token_to_text(&tokens[i]);
                result.push(text_inline);
                i += 1;
            }
        }

        Ok(result)
    }

    /// Level 1: Try to match a delimiter at the given position
    fn try_match_at(&self, tokens: &[ScannerToken], start: usize) -> Option<(SpanMatch, String)> {
        for matcher in &self.matchers {
            if matcher.can_start(&tokens[start]) {
                if let Some(span) = matcher.match_span(tokens, start) {
                    return Some((span, matcher.name().to_string()));
                }
            }
        }
        None
    }

    /// Level 2: Classify a matched span
    fn classify_span(
        &self,
        span: SpanMatch,
        _matcher_name: &str,
    ) -> Result<TypedSpan, InlineParseError> {
        use crate::semantic::elements::inlines::level2_classifiers::{
            FormattingClassifier, ReferenceTypeClassifier,
        };

        let inline_type = if span.matcher_name == "reference" {
            // Use reference classifier for references
            let classifier = ReferenceTypeClassifier::new();
            classifier.classify(&span)?
        } else {
            // Use formatting classifier for formatting elements
            let classifier = FormattingClassifier;
            classifier.classify(&span)?
        };

        Ok(TypedSpan { span, inline_type })
    }

    /// Level 3: Process a typed span into AST
    fn process_span(&self, typed_span: TypedSpan) -> Result<Inline, InlineParseError> {
        use crate::semantic::elements::inlines::level3_processors::get_processor;

        let processor = get_processor(&typed_span.inline_type);
        processor.process(&typed_span)
    }

    /// Convert a token to plain text inline
    fn token_to_text(&self, token: &ScannerToken) -> Inline {
        let token_sequence = ScannerTokenSequence {
            tokens: vec![token.clone()],
        };
        Inline::TextLine(TextTransform::Identity(
            crate::ast::elements::formatting::inlines::Text::simple_with_tokens(
                token.content(),
                token_sequence,
            ),
        ))
    }
}

impl Default for InlinePipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a fully configured inline pipeline with all standard matchers
///
/// This creates a pipeline with all built-in inline element matchers in the
/// correct priority order:
/// 1. Code (highest priority - prevents conflicts)
/// 2. Math
/// 3. References (all types: citations, footnotes, etc.)
/// 4. Bold
/// 5. Italic
pub fn create_standard_pipeline() -> InlinePipeline {
    use crate::semantic::elements::inlines::level1_matchers::*;

    InlinePipeline::new()
        .with_matcher(Box::new(code_matcher()))
        .with_matcher(Box::new(math_matcher()))
        .with_matcher(Box::new(reference_matcher()))
        .with_matcher(Box::new(bold_matcher()))
        .with_matcher(Box::new(italic_matcher()))
}

/// Convert Vec<Inline> to Vec<TextTransform> for backward compatibility
///
/// This helper function extracts TextTransform elements from Inline::TextLine variants.
/// Reference elements are currently converted to plain text since the ParagraphBlock
/// structure doesn't yet support mixed Inline content.
///
/// TODO: Update ParagraphBlock.content to Vec<Inline> to properly support references
pub fn inlines_to_text_transforms(inlines: Vec<Inline>) -> Vec<TextTransform> {
    inlines
        .into_iter()
        .map(|inline| match inline {
            Inline::TextLine(transform) => transform,
            Inline::Reference(reference) => {
                // Convert reference to plain text for now
                // Eventually ParagraphBlock should support Vec<Inline>
                let text = reference.target.display_text();
                TextTransform::Identity(Text::simple_with_tokens(&text, reference.tokens))
            }
            Inline::Link { target, tokens, .. } => {
                // Convert link to plain text for now
                TextTransform::Identity(Text::simple_with_tokens(&target, tokens))
            }
            Inline::Custom { name, tokens, .. } => {
                // Convert custom inline to plain text
                TextTransform::Identity(Text::simple_with_tokens(&name, tokens))
            }
        })
        .collect()
}
