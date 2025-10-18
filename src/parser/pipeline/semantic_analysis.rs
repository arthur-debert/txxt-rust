//! Phase 2a: Semantic Analysis
//!
//! Converts scanner tokens into semantic tokens. This is the first step
//! of Phase 2 parsing, where we elevate the low-level scanner token stream
//! into a higher-level stream of semantic nodes.
//!
//! # Overview
//!
//! Semantic analysis transforms individual scanner tokens into meaningful
//! semantic constructs while preserving structural information like
//! indentation. This phase bridges the gap between low-level tokenization
//! and high-level AST construction.
//!
//! # Key Transformations
//!
//! - **TxxtMarker**: Fundamental :: markers for structural elements
//! - **Label**: Structured identifiers for annotations and verbatim blocks
//! - **Parameters**: Key-value metadata components
//! - **SequenceMarker**: List and session numbering components
//! - **TextSpan**: Basic text content without formatting
//! - **SequenceTextLine**: Lines with sequence markers + text
//! - **PlainTextLine**: Simple text content without markers
//! - **IgnoreLine**: Raw content preserved exactly as written
//!
//! # Structural Preservation
//!
//! Structural tokens like `Indent`, `Dedent`, and `BlankLine` are passed
//! through unchanged to maintain tree structure for subsequent phases.
//!
//! # Input/Output
//!
//! - **Input**: `ScannerTokenList` from lexer (Phase 1b)
//! - **Output**: `SemanticTokenList` for AST construction (Phase 2b)

use crate::ast::scanner_tokens::{Position, ScannerToken, SequenceMarkerType, SourceSpan};
use crate::ast::semantic_tokens::{
    SemanticNumberingForm, SemanticNumberingStyle, SemanticToken, SemanticTokenBuilder,
    SemanticTokenList,
};

/// Semantic analysis parser for converting scanner tokens to semantic tokens
///
/// This parser takes a flat stream of scanner tokens and transforms them
/// into higher-level semantic tokens that represent syntactic constructs.
pub struct SemanticAnalyzer;

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer instance
    pub fn new() -> Self {
        Self
    }

    /// Analyze scanner tokens and convert them to semantic tokens
    ///
    /// Takes a flat stream of scanner tokens and transforms them into
    /// semantic tokens that represent higher-level syntactic constructs.
    /// Structural tokens are passed through unchanged.
    ///
    /// # Arguments
    /// * `scanner_tokens` - The scanner token vector from Phase 1b
    ///
    /// # Returns
    /// * `Result<SemanticTokenList, SemanticAnalysisError>` - The semantic token list
    pub fn analyze(
        &self,
        scanner_tokens: Vec<ScannerToken>,
    ) -> Result<SemanticTokenList, SemanticAnalysisError> {
        let mut semantic_tokens = Vec::new();
        let mut i = 0;

        while i < scanner_tokens.len() {
            let token = &scanner_tokens[i];

            match token {
                // Structural tokens - pass through unchanged
                ScannerToken::BlankLine { span, .. } => {
                    semantic_tokens.push(SemanticToken::BlankLine { span: span.clone() });
                }
                ScannerToken::Indent { span } => {
                    semantic_tokens.push(SemanticToken::Indent { span: span.clone() });
                }
                ScannerToken::Dedent { span } => {
                    semantic_tokens.push(SemanticToken::Dedent { span: span.clone() });
                }

                // TxxtMarker transformation - Issue #81
                ScannerToken::TxxtMarker { .. } => {
                    semantic_tokens.push(self.transform_txxt_marker(token)?);
                }

                // Label transformation - Issue #82
                ScannerToken::Identifier { content, span } => {
                    semantic_tokens.push(self.transform_label(content.clone(), span.clone())?);
                }

                // Text Span transformation - Issue #85
                ScannerToken::Text { content, span } => {
                    semantic_tokens.push(self.transform_text_span(content.clone(), span.clone())?);
                }

                // Sequence Marker transformation - Issue #84
                ScannerToken::SequenceMarker { marker_type, span } => {
                    semantic_tokens
                        .push(self.transform_sequence_marker(marker_type.clone(), span.clone())?);
                }

                // Handle other tokens as text spans for now
                _ => {
                    // Convert other tokens to text spans as fallback
                    // This will be refined in subsequent transformation issues
                    let content = self.token_to_text_content(token);
                    semantic_tokens.push(SemanticTokenBuilder::text_span(
                        content,
                        token.span().clone(),
                    ));
                }
            }

            i += 1;
        }

        Ok(SemanticTokenList::with_tokens(semantic_tokens))
    }

    /// Transform TxxtMarker scanner token to semantic token
    ///
    /// This implements the TxxtMarker transformation as specified in Issue #81.
    /// TxxtMarker tokens represent the fundamental :: markers used across
    /// annotations, definitions, and verbatim blocks.
    ///
    /// # Arguments
    /// * `token` - The TxxtMarker scanner token
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_txxt_marker(
        &self,
        token: &ScannerToken,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        match token {
            ScannerToken::TxxtMarker { span } => {
                // Transform TxxtMarker scanner token to TxxtMarker semantic token
                // This preserves the fundamental :: marker information for use
                // in subsequent parsing phases
                Ok(SemanticTokenBuilder::txxt_marker(span.clone()))
            }
            _ => Err(SemanticAnalysisError::InvalidTokenType {
                expected: "TxxtMarker".to_string(),
                actual: format!("{:?}", token),
            }),
        }
    }

    /// Transform Identifier scanner token to Label semantic token
    ///
    /// This implements the Label transformation as specified in Issue #82.
    /// Label tokens represent structured identifiers used in annotations and
    /// verbatim blocks, supporting namespaced identifiers like "python",
    /// "org.example.custom".
    ///
    /// # Arguments
    /// * `content` - The identifier content from the scanner token
    /// * `span` - The source span of the identifier
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_label(
        &self,
        content: String,
        span: SourceSpan,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        // Validate that the content is a valid label
        if content.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Label content cannot be empty".to_string(),
            ));
        }

        // Check if the label starts with a valid character
        if let Some(first_char) = content.chars().next() {
            if !self.is_valid_label_start(first_char) {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Label must start with a letter, got '{}'",
                    first_char
                )));
            }
        }

        // Validate all characters in the label
        for (i, c) in content.chars().enumerate() {
            if !self.is_valid_label_char(c) && c != '.' {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Invalid character '{}' at position {} in label '{}'",
                    c, i, content
                )));
            }
        }

        // Transform Identifier scanner token to Label semantic token
        Ok(SemanticTokenBuilder::label(content, span))
    }

    /// Check if a character is valid at the start of a label
    pub fn is_valid_label_start(&self, c: char) -> bool {
        c.is_ascii_alphabetic()
    }

    /// Check if a character is valid within a label
    pub fn is_valid_label_char(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }

    /// Transform Text scanner token to TextSpan semantic token
    ///
    /// This implements the Text Span transformation as specified in Issue #85.
    /// TextSpan tokens represent basic text content without special formatting,
    /// serving as building blocks for larger line constructs.
    ///
    /// # Arguments
    /// * `content` - The text content from the scanner token
    /// * `span` - The source span of the text
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_text_span(
        &self,
        content: String,
        span: SourceSpan,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        // Validate that the content is not empty
        if content.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Text span content cannot be empty".to_string(),
            ));
        }

        // Transform Text scanner token to TextSpan semantic token
        // This preserves the basic text content for use in subsequent parsing phases
        Ok(SemanticTokenBuilder::text_span(content, span))
    }

    /// Transform SequenceMarker scanner token to SequenceMarker semantic token
    ///
    /// This implements the Sequence Marker transformation as specified in Issue #84.
    /// SequenceMarker tokens represent list and session numbering components,
    /// handling numeric (1.), alphabetic (a.), roman (i.), and plain (-) markers,
    /// in both regular (2.) and extended (1.3.b) forms.
    ///
    /// # Arguments
    /// * `marker_type` - The sequence marker type from the scanner token
    /// * `span` - The source span of the marker
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_sequence_marker(
        &self,
        marker_type: SequenceMarkerType,
        span: SourceSpan,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        let (style, form) = self.classify_sequence_marker(&marker_type);
        let marker_text = marker_type.content().to_string();

        // Transform SequenceMarker scanner token to SequenceMarker semantic token
        Ok(SemanticTokenBuilder::sequence_marker(
            style,
            form,
            marker_text,
            span,
        ))
    }

    /// Classify a sequence marker type into semantic numbering style and form
    ///
    /// This helper method determines the semantic numbering style and form
    /// based on the scanner token's sequence marker type.
    ///
    /// # Arguments
    /// * `marker_type` - The sequence marker type to classify
    ///
    /// # Returns
    /// * `(SemanticNumberingStyle, SemanticNumberingForm)` - The classified style and form
    pub fn classify_sequence_marker(
        &self,
        marker_type: &SequenceMarkerType,
    ) -> (SemanticNumberingStyle, SemanticNumberingForm) {
        match marker_type {
            SequenceMarkerType::Plain(_) => (
                SemanticNumberingStyle::Plain,
                SemanticNumberingForm::Regular,
            ),
            SequenceMarkerType::Numerical(_, _) => (
                SemanticNumberingStyle::Numeric,
                SemanticNumberingForm::Regular,
            ),
            SequenceMarkerType::Alphabetical(_, _) => (
                SemanticNumberingStyle::Alphabetic,
                SemanticNumberingForm::Regular,
            ),
            SequenceMarkerType::Roman(_, _) => (
                SemanticNumberingStyle::Roman,
                SemanticNumberingForm::Regular,
            ),
        }
    }

    /// Transform a sequence of text tokens into a PlainTextLine semantic token
    /// PlainTextLine tokens represent simple text content without special markers
    /// or structure, containing a single TextSpan component.
    ///
    /// # Arguments
    /// * `text_tokens` - Vector of Text scanner tokens that form a line
    /// * `line_span` - The source span covering the entire line
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_plain_text_line(
        &self,
        text_tokens: Vec<ScannerToken>,
        line_span: SourceSpan,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        // Validate that we have at least one text token
        if text_tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Plain text line must contain at least one text token".to_string(),
            ));
        }

        // Validate that all tokens are Text tokens
        for token in &text_tokens {
            if !matches!(token, ScannerToken::Text { .. }) {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Plain text line can only contain Text tokens, got {:?}",
                    token
                )));
            }
        }

        // Combine all text content into a single string
        let combined_content = text_tokens
            .iter()
            .map(|token| {
                if let ScannerToken::Text { content, .. } = token {
                    content.as_str()
                } else {
                    "" // This should never happen due to validation above
                }
            })
            .collect::<Vec<&str>>()
            .join("");

        // Create a single TextSpan for the combined content
        let text_span = SemanticTokenBuilder::text_span(combined_content, line_span.clone());

        // Transform to PlainTextLine semantic token
        Ok(SemanticTokenBuilder::plain_text_line(text_span, line_span))
    }

    /// Transform a sequence marker followed by text content into a SequenceTextLine semantic token
    ///
    /// This implements the Sequence Text Line transformation as specified in Issue #86.
    /// SequenceTextLine tokens represent lines beginning with sequence markers followed
    /// by text content, combining Sequence Marker and Text Span components.
    ///
    /// # Arguments
    /// * `marker_token` - The sequence marker semantic token
    /// * `text_tokens` - Vector of Text scanner tokens that form the content
    /// * `line_span` - The source span covering the entire line
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_sequence_text_line(
        &self,
        marker_token: SemanticToken,
        text_tokens: Vec<ScannerToken>,
        line_span: SourceSpan,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        // Validate that we have at least one text token
        if text_tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Sequence text line must contain at least one text token".to_string(),
            ));
        }

        // Validate that all tokens are Text tokens
        for token in &text_tokens {
            if !matches!(token, ScannerToken::Text { .. }) {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Sequence text line can only contain Text tokens, got {:?}",
                    token
                )));
            }
        }

        // Validate that the marker token is a SequenceMarker
        if !matches!(marker_token, SemanticToken::SequenceMarker { .. }) {
            return Err(SemanticAnalysisError::AnalysisError(format!(
                "Sequence text line marker must be a SequenceMarker token, got {:?}",
                marker_token
            )));
        }

        // Combine all text content into a single string
        let combined_content = text_tokens
            .iter()
            .map(|token| {
                if let ScannerToken::Text { content, .. } = token {
                    content.as_str()
                } else {
                    "" // This should never happen due to validation above
                }
            })
            .collect::<Vec<&str>>()
            .join("");

        // Create a single TextSpan for the combined content
        let text_span = SemanticTokenBuilder::text_span(combined_content, line_span.clone());

        // Transform to SequenceTextLine semantic token
        Ok(SemanticTokenBuilder::sequence_text_line(
            marker_token,
            text_span,
            line_span,
        ))
    }

    /// Transform scanner tokens into an Annotation semantic token
    ///
    /// This implements the Annotation transformation as specified in Issue #88.
    /// Annotation tokens represent metadata elements that attach structured information
    /// to other elements. They follow the pattern:
    /// TxxtMarker + Whitespace + Text (label) + Whitespace + TxxtMarker + Text? (content)
    ///
    /// # Arguments
    /// * `tokens` - Vector of scanner tokens that form an annotation
    /// * `span` - The source span covering the entire annotation
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_annotation(
        &self,
        tokens: Vec<ScannerToken>,
        span: SourceSpan,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        // Validate minimum annotation structure: :: label ::
        if tokens.len() < 5 {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation must have at least 5 tokens: :: label ::".to_string(),
            ));
        }

        // Check for opening TxxtMarker
        if !matches!(tokens[0], ScannerToken::TxxtMarker { .. }) {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation must start with TxxtMarker".to_string(),
            ));
        }

        // Check for closing TxxtMarker (should be at position 4 for basic annotation)
        let closing_marker_pos = self.find_closing_txxt_marker(&tokens)?;
        if closing_marker_pos < 4 {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation must have proper structure: :: label ::".to_string(),
            ));
        }

        // Extract label (between first TxxtMarker and closing TxxtMarker)
        let label_tokens = &tokens[2..closing_marker_pos];
        let (label_token, parameters) = self.parse_label_with_parameters(label_tokens)?;

        // Extract content (after closing TxxtMarker, if any)
        let content = if closing_marker_pos + 1 < tokens.len() {
            let content_tokens = &tokens[closing_marker_pos + 1..];
            Some(self.parse_annotation_content(content_tokens)?)
        } else {
            None
        };

        // Transform to Annotation semantic token
        Ok(SemanticTokenBuilder::annotation(
            label_token,
            parameters,
            content,
            span,
        ))
    }

    /// Transform scanner tokens into a Definition semantic token
    ///
    /// This implements the Definition transformation as specified in Issue #88.
    /// Definition tokens represent structured elements that define terms, concepts,
    /// and entities. They follow the pattern:
    /// Text + Whitespace + TxxtMarker
    ///
    /// # Arguments
    /// * `tokens` - Vector of scanner tokens that form a definition
    /// * `span` - The source span covering the entire definition
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_definition(
        &self,
        tokens: Vec<ScannerToken>,
        span: SourceSpan,
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        // Validate minimum definition structure: term ::
        if tokens.len() < 3 {
            return Err(SemanticAnalysisError::AnalysisError(
                "Definition must have at least 3 tokens: term ::".to_string(),
            ));
        }

        // Check for closing TxxtMarker (should be at the end)
        let txxt_marker_pos = tokens.len() - 1;
        if !matches!(tokens[txxt_marker_pos], ScannerToken::TxxtMarker { .. }) {
            return Err(SemanticAnalysisError::AnalysisError(
                "Definition must end with TxxtMarker".to_string(),
            ));
        }

        // Extract term (everything before the final TxxtMarker)
        let term_tokens = &tokens[..txxt_marker_pos];
        let (term_token, parameters) = self.parse_definition_term_with_parameters(term_tokens)?;

        // Transform to Definition semantic token
        Ok(SemanticTokenBuilder::definition(
            term_token, parameters, span,
        ))
    }

    /// Find the position of the closing TxxtMarker in an annotation
    ///
    /// This helper method searches for the second TxxtMarker in an annotation,
    /// which marks the end of the label section.
    ///
    /// # Arguments
    /// * `tokens` - The scanner tokens to search
    ///
    /// # Returns
    /// * `Result<usize, SemanticAnalysisError>` - The position of the closing marker
    fn find_closing_txxt_marker(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<usize, SemanticAnalysisError> {
        let mut marker_count = 0;
        for (i, token) in tokens.iter().enumerate() {
            if matches!(token, ScannerToken::TxxtMarker { .. }) {
                marker_count += 1;
                if marker_count == 2 {
                    return Ok(i);
                }
            }
        }
        Err(SemanticAnalysisError::AnalysisError(
            "Annotation must have closing TxxtMarker".to_string(),
        ))
    }

    /// Parse label tokens and extract parameters if present
    ///
    /// This helper method processes label tokens and separates the main label
    /// from any parameters (key=value pairs).
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing the label (may include parameters)
    ///
    /// # Returns
    /// * `Result<(SemanticToken, Option<SemanticToken>), SemanticAnalysisError>` - (label, parameters)
    fn parse_label_with_parameters(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<(SemanticToken, Option<SemanticToken>), SemanticAnalysisError> {
        // Look for colon separator to identify parameters
        let colon_pos = tokens
            .iter()
            .position(|token| matches!(token, ScannerToken::Colon { .. }));

        if let Some(pos) = colon_pos {
            // Split into label and parameters
            let label_tokens = &tokens[..pos];
            let param_tokens = &tokens[pos + 1..];

            // Create label semantic token
            let label_token = self.tokens_to_text_span(label_tokens)?;

            // Create parameters semantic token if there are parameter tokens
            let parameters = if param_tokens.is_empty()
                || param_tokens
                    .iter()
                    .all(|token| matches!(token, ScannerToken::Whitespace { .. }))
            {
                None
            } else {
                Some(self.parse_parameters(param_tokens)?)
            };

            Ok((label_token, parameters))
        } else {
            // No parameters, just create label
            let label_token = self.tokens_to_text_span(tokens)?;
            Ok((label_token, None))
        }
    }

    /// Parse definition term tokens and extract parameters if present
    ///
    /// This helper method processes definition term tokens and separates the main term
    /// from any parameters (key=value pairs). Unlike labels, definition terms preserve
    /// whitespace to maintain formatting like verbatim titles.
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing the definition term (may include parameters)
    ///
    /// # Returns
    /// * `Result<(SemanticToken, Option<SemanticToken>), SemanticAnalysisError>` - (term, parameters)
    fn parse_definition_term_with_parameters(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<(SemanticToken, Option<SemanticToken>), SemanticAnalysisError> {
        // Look for colon separator to identify parameters
        let colon_pos = tokens
            .iter()
            .position(|token| matches!(token, ScannerToken::Colon { .. }));

        if let Some(pos) = colon_pos {
            // Split into term and parameters
            let term_tokens = &tokens[..pos];
            let param_tokens = &tokens[pos + 1..];

            // Create term semantic token (preserve whitespace)
            let term_token = self.tokens_to_text_span_preserve_whitespace(term_tokens)?;

            // Create parameters semantic token if there are parameter tokens
            let parameters = if param_tokens.is_empty()
                || param_tokens
                    .iter()
                    .all(|token| matches!(token, ScannerToken::Whitespace { .. }))
            {
                None
            } else {
                Some(self.parse_parameters(param_tokens)?)
            };

            Ok((term_token, parameters))
        } else {
            // No parameters, just create term (preserve whitespace)
            let term_token = self.tokens_to_text_span_preserve_whitespace(tokens)?;
            Ok((term_token, None))
        }
    }

    /// Parse parameter tokens into a Parameters semantic token
    ///
    /// This helper method processes parameter tokens and creates a structured
    /// Parameters semantic token with key-value pairs.
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing parameters
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The parameters semantic token
    fn parse_parameters(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        // For now, create a simple parameters token with the raw content
        // This will be enhanced in future iterations to properly parse key=value pairs
        // Filter out whitespace for parameters to create clean key=value pairs
        let content = tokens
            .iter()
            .filter(|token| !matches!(token, ScannerToken::Whitespace { .. }))
            .map(|token| token.content())
            .collect::<Vec<&str>>()
            .join("");

        // Create a simple parameters map (placeholder implementation)
        let mut params = std::collections::HashMap::new();
        params.insert("raw".to_string(), content);

        // Calculate span for the parameters
        let span = if tokens.is_empty() {
            SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 0 },
            }
        } else {
            SourceSpan {
                start: tokens[0].span().start,
                end: tokens[tokens.len() - 1].span().end,
            }
        };

        Ok(SemanticTokenBuilder::parameters(params, span))
    }

    /// Parse annotation content tokens into a semantic token
    ///
    /// This helper method processes content tokens after the closing TxxtMarker
    /// and creates an appropriate semantic token for the content.
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing the annotation content
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The content semantic token
    fn parse_annotation_content(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        if tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation content cannot be empty".to_string(),
            ));
        }

        // For content, preserve whitespace to maintain formatting
        // This will be enhanced in future iterations to handle complex content
        self.tokens_to_text_span_preserve_whitespace(tokens)
    }

    /// Convert a sequence of tokens to a TextSpan semantic token preserving whitespace
    ///
    /// This helper method combines multiple tokens into a single TextSpan,
    /// preserving whitespace for content that needs formatting.
    ///
    /// # Arguments
    /// * `tokens` - The tokens to combine
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The text span semantic token
    fn tokens_to_text_span_preserve_whitespace(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        if tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Cannot create text span from empty tokens".to_string(),
            ));
        }

        // Preserve whitespace for content but trim leading/trailing whitespace
        let content = tokens
            .iter()
            .map(|token| token.content())
            .collect::<Vec<&str>>()
            .join("")
            .trim()
            .to_string();

        let span = SourceSpan {
            start: tokens[0].span().start,
            end: tokens[tokens.len() - 1].span().end,
        };

        Ok(SemanticTokenBuilder::text_span(content, span))
    }

    /// Convert a sequence of tokens to a TextSpan semantic token
    ///
    /// This helper method combines multiple tokens into a single TextSpan,
    /// preserving the source span information. It filters out whitespace tokens
    /// to create clean text content.
    ///
    /// # Arguments
    /// * `tokens` - The tokens to combine
    ///
    /// # Returns
    /// * `Result<SemanticToken, SemanticAnalysisError>` - The text span semantic token
    fn tokens_to_text_span(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<SemanticToken, SemanticAnalysisError> {
        if tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Cannot create text span from empty tokens".to_string(),
            ));
        }

        // Filter out whitespace tokens and combine content
        let content = tokens
            .iter()
            .filter(|token| !matches!(token, ScannerToken::Whitespace { .. }))
            .map(|token| token.content())
            .collect::<Vec<&str>>()
            .join("");

        let span = SourceSpan {
            start: tokens[0].span().start,
            end: tokens[tokens.len() - 1].span().end,
        };

        Ok(SemanticTokenBuilder::text_span(content, span))
    }

    /// This is a utility method to convert any scanner token to text content
    /// when we don't have a specific transformation for it yet.
    fn token_to_text_content(&self, token: &ScannerToken) -> String {
        match token {
            ScannerToken::Text { content, .. } => content.clone(),
            ScannerToken::Whitespace { content, .. } => content.clone(),
            ScannerToken::Newline { .. } => "\n".to_string(),
            ScannerToken::Dash { .. } => "-".to_string(),
            ScannerToken::Period { .. } => ".".to_string(),
            ScannerToken::LeftBracket { .. } => "[".to_string(),
            ScannerToken::RightBracket { .. } => "]".to_string(),
            ScannerToken::AtSign { .. } => "@".to_string(),
            ScannerToken::LeftParen { .. } => "(".to_string(),
            ScannerToken::RightParen { .. } => ")".to_string(),
            ScannerToken::Colon { .. } => ":".to_string(),
            ScannerToken::Equals { .. } => "=".to_string(),
            ScannerToken::Comma { .. } => ",".to_string(),
            ScannerToken::TxxtMarker { .. } => "::".to_string(),
            ScannerToken::Identifier { content, .. } => content.clone(),
            ScannerToken::SequenceMarker { marker_type, .. } => {
                // Convert sequence marker to its text representation
                marker_type.content().to_string()
            }
            _ => "".to_string(),
        }
    }
}

/// Errors that can occur during semantic analysis
#[derive(Debug)]
pub enum SemanticAnalysisError {
    /// Invalid token type encountered
    InvalidTokenType { expected: String, actual: String },
    /// General semantic analysis error
    AnalysisError(String),
}

impl std::fmt::Display for SemanticAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticAnalysisError::InvalidTokenType { expected, actual } => {
                write!(
                    f,
                    "Invalid token type: expected {}, got {}",
                    expected, actual
                )
            }
            SemanticAnalysisError::AnalysisError(msg) => {
                write!(f, "Semantic analysis error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SemanticAnalysisError {}
