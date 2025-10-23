//! Generic Registration-Based Inline Parsing Engine
//!
//! This module provides a flexible, extensible inline parsing system where
//! inline types register themselves with the engine and define their own
//! processing pipelines.
//!
//! # Architecture
//!
//! - **PipelineData**: Common trait for all data flowing through pipelines
//! - **Stage**: Transform or dispatch operations
//! - **Pipeline**: Sequence of stages
//! - **InlineEngine**: Orchestrates registration, validation, and execution
//!
//! # Key Features
//!
//! - Registration-based: Add inline types without modifying engine code
//! - Arbitrary pipeline length: Not constrained to fixed number of levels
//! - Type-based dispatch: Branch to different processing based on classification
//! - Delimiter validation: Ensures uniqueness across all registered inlines
//! - Fallthrough error handling: Malformed syntax becomes plain text, never crashes
//!
//! # Example Usage
//!
//! ```ignore
//! let mut engine = InlineEngine::new();
//!
//! // Register a simple inline type
//! engine.register(InlineDefinition {
//!     name: "bold",
//!     delimiters: DelimiterSpec { start: '*', end: '*' },
//!     pipeline: PipelineBuilder::new()
//!         .then("parse", parse_bold_content)
//!         .then("build", build_bold_inline)
//!         .build(),
//! })?;
//!
//! // Parse tokens
//! let inlines = engine.parse(&tokens);
//! ```

pub mod pipeline;
pub mod pipeline_data;
pub mod reference_example;

pub use pipeline::{Pipeline, PipelineBuilder, Stage, StageResult, TransformFn, TypeKeyFn};
pub use pipeline_data::{ClassifiedSpan, MatchedSpan, PipelineData, StageData, StageError};

use crate::ast::elements::formatting::inlines::Inline;
use crate::cst::ScannerToken;

/// Single-character delimiter specification
///
/// Delimiters are restricted to single characters for simplicity and
/// to avoid ambiguous overlap (e.g., `[` vs `[[`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DelimiterSpec {
    /// Start delimiter character
    pub start: char,

    /// End delimiter character
    pub end: char,
}

impl DelimiterSpec {
    /// Create a new delimiter spec
    pub fn new(start: char, end: char) -> Self {
        Self { start, end }
    }

    /// Check if a token matches the start delimiter
    pub fn matches_start(&self, token: &ScannerToken) -> bool {
        match token {
            ScannerToken::Text { content, .. } if content.len() == 1 => {
                content.starts_with(self.start)
            }
            _ => false,
        }
    }

    /// Check if a token matches the end delimiter
    pub fn matches_end(&self, token: &ScannerToken) -> bool {
        match token {
            ScannerToken::Text { content, .. } if content.len() == 1 => {
                content.starts_with(self.end)
            }
            _ => false,
        }
    }
}

/// Inline type definition for registration
///
/// Each inline type defines its delimiters and processing pipeline.
pub struct InlineDefinition {
    /// Unique name for this inline type
    pub name: &'static str,

    /// Delimiter specification
    pub delimiters: DelimiterSpec,

    /// Processing pipeline
    pub pipeline: Pipeline,
}

/// Errors that can occur during registration or parsing
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// Duplicate delimiters detected during registration
    DuplicateDelimiters {
        existing: &'static str,
        new: &'static str,
        delimiters: DelimiterSpec,
    },

    /// Empty pipeline in definition
    EmptyPipeline { name: &'static str },

    /// Pipeline execution error
    PipelineError {
        inline_name: &'static str,
        stage_name: &'static str,
        error: StageError,
    },

    /// Pipeline did not produce Inline
    InvalidOutput {
        inline_name: &'static str,
        actual_type: &'static str,
    },
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::DuplicateDelimiters {
                existing,
                new,
                delimiters,
            } => {
                write!(
                    f,
                    "Duplicate delimiters {:?} between '{}' and '{}'",
                    delimiters, existing, new
                )
            }
            EngineError::EmptyPipeline { name } => {
                write!(f, "Empty pipeline for inline type '{}'", name)
            }
            EngineError::PipelineError {
                inline_name,
                stage_name,
                error,
            } => {
                write!(
                    f,
                    "Pipeline error in '{}' at stage '{}': {}",
                    inline_name, stage_name, error
                )
            }
            EngineError::InvalidOutput {
                inline_name,
                actual_type,
            } => {
                write!(
                    f,
                    "Pipeline for '{}' did not produce Inline (got {})",
                    inline_name, actual_type
                )
            }
        }
    }
}

impl std::error::Error for EngineError {}

/// Generic inline parsing engine
///
/// Manages registration of inline types and orchestrates parsing.
pub struct InlineEngine {
    definitions: Vec<InlineDefinition>,
}

impl InlineEngine {
    /// Create a new empty engine
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
        }
    }

    /// Register a new inline type
    ///
    /// # Validation
    ///
    /// - Checks delimiter uniqueness against all registered inlines
    /// - Validates pipeline is non-empty
    ///
    /// # Errors
    ///
    /// Returns error if delimiters conflict or pipeline is invalid
    pub fn register(&mut self, definition: InlineDefinition) -> Result<(), EngineError> {
        // Validate delimiter uniqueness
        for existing in &self.definitions {
            if existing.delimiters == definition.delimiters {
                return Err(EngineError::DuplicateDelimiters {
                    existing: existing.name,
                    new: definition.name,
                    delimiters: definition.delimiters,
                });
            }
        }

        // Validate pipeline is non-empty
        if definition.pipeline.is_empty() {
            return Err(EngineError::EmptyPipeline {
                name: definition.name,
            });
        }

        self.definitions.push(definition);
        Ok(())
    }

    /// Get number of registered inline types
    pub fn registered_count(&self) -> usize {
        self.definitions.len()
    }

    /// Parse tokens into inline elements
    ///
    /// Iterates through the token stream, attempting to match registered
    /// inline types. On match, executes the inline's pipeline. On error,
    /// falls through to plain text with warning log.
    ///
    /// # Error Handling
    ///
    /// Parse errors never cause hard failures. Instead:
    /// - Log warning with position
    /// - Render problematic span as plain text
    /// - Continue parsing
    pub fn parse(&self, tokens: &[ScannerToken]) -> Vec<Inline> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            match self.try_match_at(tokens, i) {
                Ok(Some((inline, end))) => {
                    result.push(inline);
                    i = end;
                }
                Ok(None) => {
                    // No match - plain text
                    result.push(self.token_to_inline(&tokens[i]));
                    i += 1;
                }
                Err(e) => {
                    // Error - fallthrough to plain text with warning
                    // TODO: Add proper logging when log infrastructure is available
                    eprintln!("Inline parsing error at position {}: {}", i, e);
                    result.push(self.token_to_inline(&tokens[i]));
                    i += 1;
                }
            }
        }

        result
    }

    /// Try to match an inline at the given position
    ///
    /// Returns Some((inline, end_position)) on successful match, None if no match
    fn try_match_at(
        &self,
        tokens: &[ScannerToken],
        start: usize,
    ) -> Result<Option<(Inline, usize)>, EngineError> {
        for def in &self.definitions {
            if def.delimiters.matches_start(&tokens[start]) {
                if let Some(span) = self.match_span(tokens, start, &def.delimiters, def.name)? {
                    let inline = self.run_pipeline(&span, &def.pipeline, def.name)?;
                    return Ok(Some((inline, span.end)));
                }
            }
        }
        Ok(None)
    }

    /// Match delimiter span
    fn match_span(
        &self,
        tokens: &[ScannerToken],
        start: usize,
        delimiters: &DelimiterSpec,
        inline_name: &'static str,
    ) -> Result<Option<MatchedSpan>, EngineError> {
        // Find closing delimiter
        let mut end = None;
        for (i, token) in tokens[start + 1..].iter().enumerate() {
            // Check for newlines (single-line constraint)
            if matches!(
                token,
                ScannerToken::Newline { .. } | ScannerToken::BlankLine { .. }
            ) {
                return Ok(None);
            }

            if delimiters.matches_end(token) {
                end = Some(start + 1 + i);
                break;
            }
        }

        if let Some(end_pos) = end {
            let inner_tokens = tokens[start + 1..end_pos].to_vec();

            // Must have content
            if inner_tokens.is_empty() {
                return Ok(None);
            }

            Ok(Some(MatchedSpan {
                inner_tokens,
                full_tokens: tokens[start..=end_pos].to_vec(),
                start,
                end: end_pos + 1,
                inline_name: inline_name.to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Execute a pipeline
    fn run_pipeline(
        &self,
        span: &MatchedSpan,
        pipeline: &Pipeline,
        inline_name: &'static str,
    ) -> Result<Inline, EngineError> {
        let mut data = StageData::new(span.clone());

        for stage in &pipeline.stages {
            data = self.run_stage(data, stage, inline_name).map_err(|error| {
                EngineError::PipelineError {
                    inline_name,
                    stage_name: stage.name(),
                    error,
                }
            })?;
        }

        // Final data must be Inline
        data.downcast::<Inline>()
            .cloned()
            .map_err(|_| EngineError::InvalidOutput {
                inline_name,
                actual_type: data.type_name(),
            })
    }

    /// Execute a single stage
    #[allow(clippy::only_used_in_recursion)]
    fn run_stage(
        &self,
        data: StageData,
        stage: &Stage,
        inline_name: &'static str,
    ) -> Result<StageData, StageError> {
        match stage {
            Stage::Transform { func, .. } => func(data),

            Stage::Dispatch {
                type_fn,
                branches,
                default,
                ..
            } => {
                let type_key = type_fn(&data);

                let sub_pipeline = branches.get(&type_key).unwrap_or(default);

                // Run sub-pipeline stages
                let mut current = data;
                for sub_stage in &sub_pipeline.stages {
                    current = self.run_stage(current, sub_stage, inline_name)?;
                }

                Ok(current)
            }
        }
    }

    /// Convert token to plain text inline
    fn token_to_inline(&self, token: &ScannerToken) -> Inline {
        use crate::ast::elements::formatting::inlines::{Text, TextTransform};
        use crate::cst::ScannerTokenSequence;

        let token_sequence = ScannerTokenSequence {
            tokens: vec![token.clone()],
        };

        Inline::TextLine(TextTransform::Identity(Text::simple_with_tokens(
            token.content(),
            token_sequence,
        )))
    }
}

impl Default for InlineEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delimiter_spec_equality() {
        let spec1 = DelimiterSpec::new('*', '*');
        let spec2 = DelimiterSpec::new('*', '*');
        let spec3 = DelimiterSpec::new('[', ']');

        assert_eq!(spec1, spec2);
        assert_ne!(spec1, spec3);
    }

    #[test]
    fn test_engine_registration_validates_uniqueness() {
        let mut engine = InlineEngine::new();

        let def1 = InlineDefinition {
            name: "bold",
            delimiters: DelimiterSpec::new('*', '*'),
            pipeline: PipelineBuilder::new().then("test", Ok).build(),
        };

        assert!(engine.register(def1).is_ok());
        assert_eq!(engine.registered_count(), 1);

        // Try to register another inline with same delimiters
        let def2 = InlineDefinition {
            name: "italic",
            delimiters: DelimiterSpec::new('*', '*'), // Same as bold
            pipeline: PipelineBuilder::new().then("test", Ok).build(),
        };

        let result = engine.register(def2);
        assert!(result.is_err());

        match result {
            Err(EngineError::DuplicateDelimiters { existing, new, .. }) => {
                assert_eq!(existing, "bold");
                assert_eq!(new, "italic");
            }
            _ => panic!("Expected DuplicateDelimiters error"),
        }
    }

    #[test]
    fn test_engine_validates_non_empty_pipeline() {
        let mut engine = InlineEngine::new();

        let def = InlineDefinition {
            name: "test",
            delimiters: DelimiterSpec::new('*', '*'),
            pipeline: Pipeline::new(), // Empty
        };

        let result = engine.register(def);
        assert!(result.is_err());

        match result {
            Err(EngineError::EmptyPipeline { name }) => {
                assert_eq!(name, "test");
            }
            _ => panic!("Expected EmptyPipeline error"),
        }
    }

    #[test]
    fn test_engine_allows_different_delimiters() {
        let mut engine = InlineEngine::new();

        let def1 = InlineDefinition {
            name: "bold",
            delimiters: DelimiterSpec::new('*', '*'),
            pipeline: PipelineBuilder::new().then("test", Ok).build(),
        };

        let def2 = InlineDefinition {
            name: "italic",
            delimiters: DelimiterSpec::new('_', '_'),
            pipeline: PipelineBuilder::new().then("test", Ok).build(),
        };

        assert!(engine.register(def1).is_ok());
        assert!(engine.register(def2).is_ok());
        assert_eq!(engine.registered_count(), 2);
    }
}
