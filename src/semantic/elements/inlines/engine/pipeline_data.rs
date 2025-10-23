//! Pipeline Data Types and Trait
//!
//! This module defines the common interface and foundational data structures
//! for all data flowing through inline parsing pipelines.
//!
//! # Design Principles
//!
//! - **Token Preservation**: All pipeline data must provide access to source tokens
//! - **Type Flexibility**: Downcasting via Any enables arbitrary intermediate types
//! - **Position Tracking**: Token access enables error reporting with precise locations
//!
//! # Core Types
//!
//! - `PipelineData`: Common trait for all pipeline data
//! - `MatchedSpan`: Output of delimiter matching
//! - `ClassifiedSpan`: Output of type classification
//! - `StageData`: Type-erased wrapper with common interface

use crate::cst::{Position, ScannerToken};
use std::any::Any;

/// Common trait for all data flowing through inline parsing pipelines
///
/// All intermediate types in a pipeline must implement this trait to ensure:
/// - Source tokens are preserved for position tracking
/// - Type metadata is available for debugging
/// - Downcasting is possible for type-specific processing
///
/// # Examples
///
/// ```ignore
/// struct ParsedCitation {
///     entries: Vec<CitationEntry>,
///     tokens: Vec<ScannerToken>,
/// }
///
/// impl PipelineData for ParsedCitation {
///     fn tokens(&self) -> &[ScannerToken] {
///         &self.tokens
///     }
///
///     fn as_any(&self) -> &dyn Any { self }
///     fn as_any_mut(&mut self) -> &mut dyn Any { self }
/// }
/// ```
pub trait PipelineData: Send + Sync {
    /// Access to source tokens for position tracking and error reporting
    fn tokens(&self) -> &[ScannerToken];

    /// Type name for debugging and error messages
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Downcast to Any for flexible type handling
    fn as_any(&self) -> &dyn Any;

    /// Mutable downcast to Any
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Errors that can occur during stage processing
#[derive(Debug, Clone, PartialEq)]
pub enum StageError {
    /// Type mismatch during downcast
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },

    /// Invalid structure or content
    InvalidStructure(String),

    /// Missing required data
    MissingData(String),

    /// Generic processing error
    ProcessingError(String),
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            StageError::InvalidStructure(msg) => write!(f, "Invalid structure: {}", msg),
            StageError::MissingData(msg) => write!(f, "Missing data: {}", msg),
            StageError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl std::error::Error for StageError {}

/// Type-erased wrapper for pipeline data with common interface
///
/// Provides a unified interface for accessing tokens and type information
/// while allowing arbitrary concrete types via downcasting.
pub struct StageData {
    value: Box<dyn PipelineData>,
}

impl StageData {
    /// Create new stage data from any type implementing PipelineData
    pub fn new<T: PipelineData + 'static>(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }

    /// Access tokens from wrapped value (always available)
    pub fn tokens(&self) -> &[ScannerToken] {
        self.value.tokens()
    }

    /// Get start position from first token
    pub fn start_pos(&self) -> Option<Position> {
        self.tokens().first().map(|t| t.span().start)
    }

    /// Get end position from last token
    pub fn end_pos(&self) -> Option<Position> {
        self.tokens().last().map(|t| t.span().end)
    }

    /// Type name for debugging and error messages
    pub fn type_name(&self) -> &'static str {
        self.value.type_name()
    }

    /// Downcast to specific type for processing
    ///
    /// # Errors
    ///
    /// Returns TypeMismatch error if downcast fails
    pub fn downcast<T: 'static>(&self) -> Result<&T, StageError> {
        self.value
            .as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| StageError::TypeMismatch {
                expected: std::any::type_name::<T>(),
                actual: self.type_name(),
            })
    }

    /// Mutable downcast to specific type
    ///
    /// # Errors
    ///
    /// Returns TypeMismatch error if downcast fails
    pub fn downcast_mut<T: 'static>(&mut self) -> Result<&mut T, StageError> {
        let type_name = self.type_name(); // Capture before mutable borrow
        self.value
            .as_any_mut()
            .downcast_mut::<T>()
            .ok_or_else(|| StageError::TypeMismatch {
                expected: std::any::type_name::<T>(),
                actual: type_name,
            })
    }
}

/// Level 1 output: Matched delimiter span
///
/// Represents a span of tokens that has been identified by delimiter matching.
/// This is the foundational type produced by the delimiter matching stage.
#[derive(Debug, Clone)]
pub struct MatchedSpan {
    /// Tokens between delimiters (excluding delimiters themselves)
    pub inner_tokens: Vec<ScannerToken>,

    /// All tokens including delimiters
    pub full_tokens: Vec<ScannerToken>,

    /// Start position in original token stream
    pub start: usize,

    /// End position in original token stream (exclusive)
    pub end: usize,

    /// Name of the inline type that matched
    pub inline_name: String,
}

impl PipelineData for MatchedSpan {
    fn tokens(&self) -> &[ScannerToken] {
        &self.full_tokens
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Level 2 output: Classified span with type information
///
/// Represents a matched span that has been classified into a specific type.
/// This is the output of type classification stages.
#[derive(Debug, Clone)]
pub struct ClassifiedSpan {
    /// Type name determined by classification
    pub type_name: String,

    /// The matched span being classified
    pub span: MatchedSpan,
}

impl PipelineData for ClassifiedSpan {
    fn tokens(&self) -> &[ScannerToken] {
        self.span.tokens()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    fn create_test_token(content: &str) -> ScannerToken {
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
    fn test_matched_span_implements_pipeline_data() {
        let span = MatchedSpan {
            inner_tokens: vec![create_test_token("hello")],
            full_tokens: vec![
                create_test_token("["),
                create_test_token("hello"),
                create_test_token("]"),
            ],
            start: 0,
            end: 3,
            inline_name: "reference".to_string(),
        };

        assert_eq!(span.tokens().len(), 3);
        assert!(span.as_any().downcast_ref::<MatchedSpan>().is_some());
    }

    #[test]
    fn test_classified_span_implements_pipeline_data() {
        let matched = MatchedSpan {
            inner_tokens: vec![create_test_token("@citation")],
            full_tokens: vec![
                create_test_token("["),
                create_test_token("@citation"),
                create_test_token("]"),
            ],
            start: 0,
            end: 3,
            inline_name: "reference".to_string(),
        };

        let classified = ClassifiedSpan {
            type_name: "Citation".to_string(),
            span: matched,
        };

        assert_eq!(classified.tokens().len(), 3);
        assert!(classified
            .as_any()
            .downcast_ref::<ClassifiedSpan>()
            .is_some());
    }

    #[test]
    fn test_stage_data_wraps_and_unwraps() {
        let span = MatchedSpan {
            inner_tokens: vec![create_test_token("test")],
            full_tokens: vec![create_test_token("test")],
            start: 0,
            end: 1,
            inline_name: "bold".to_string(),
        };

        let stage_data = StageData::new(span.clone());

        // Can access tokens
        assert_eq!(stage_data.tokens().len(), 1);

        // Can downcast back
        let unwrapped = stage_data.downcast::<MatchedSpan>().unwrap();
        assert_eq!(unwrapped.inline_name, "bold");
    }

    #[test]
    fn test_stage_data_downcast_error() {
        let span = MatchedSpan {
            inner_tokens: vec![],
            full_tokens: vec![],
            start: 0,
            end: 0,
            inline_name: "test".to_string(),
        };

        let stage_data = StageData::new(span);

        // Trying to downcast to wrong type fails
        let result = stage_data.downcast::<ClassifiedSpan>();
        assert!(result.is_err());

        match result {
            Err(StageError::TypeMismatch { .. }) => {}
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_stage_data_position_accessors() {
        let token = create_test_token("test");
        let span = MatchedSpan {
            inner_tokens: vec![token.clone()],
            full_tokens: vec![token],
            start: 0,
            end: 1,
            inline_name: "test".to_string(),
        };

        let stage_data = StageData::new(span);

        assert!(stage_data.start_pos().is_some());
        assert!(stage_data.end_pos().is_some());

        let start = stage_data.start_pos().unwrap();
        assert_eq!(start.row, 0);
        assert_eq!(start.column, 0);
    }
}
