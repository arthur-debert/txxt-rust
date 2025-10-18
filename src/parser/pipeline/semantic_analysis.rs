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

use crate::ast::scanner_tokens::ScannerToken;
use crate::ast::semantic_tokens::{SemanticToken, SemanticTokenBuilder, SemanticTokenList};

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

                // For now, handle other tokens as text spans
                // This will be expanded in subsequent issues
                ScannerToken::Text { content, span } => {
                    semantic_tokens.push(SemanticTokenBuilder::text_span(
                        content.clone(),
                        span.clone(),
                    ));
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

    /// Convert a scanner token to text content for fallback handling
    ///
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
