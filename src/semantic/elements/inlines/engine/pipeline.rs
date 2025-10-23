//! Pipeline Stage and Execution Infrastructure
//!
//! This module defines the pipeline execution model for inline parsing.
//! Pipelines are sequences of stages that transform data from matched
//! delimiter spans to final AST nodes.
//!
//! # Stage Types
//!
//! - **Transform**: Simple function that transforms input to output
//! - **Dispatch**: Type-based branching to sub-pipelines
//!
//! # Pipeline Composition
//!
//! Pipelines can be arbitrarily composed:
//! - Simple chains: A → B → C
//! - Dispatch chains: A → (dispatch) → [B₁ → C₁] or [B₂ → C₂]
//! - Nested dispatch: Dispatch stages can contain sub-pipelines with more dispatch stages

use super::pipeline_data::{StageData, StageError};
use std::collections::HashMap;

/// Result type for stage processing
pub type StageResult = Result<StageData, StageError>;

/// Function signature for transform stages
pub type TransformFn = fn(StageData) -> StageResult;

/// Function signature for extracting dispatch type key
pub type TypeKeyFn = fn(&StageData) -> String;

/// Pipeline stage - can be a transformation or type-based dispatch
///
/// # Transform Stage
///
/// A simple function that transforms input data to output data.
/// Both input and output must implement PipelineData.
///
/// # Dispatch Stage
///
/// Extracts a type key from input data and executes the corresponding
/// sub-pipeline. Requires a default branch for unknown type keys.
///
/// Dispatch stages support nesting - a sub-pipeline can contain another
/// dispatch stage, enabling complex type-based processing trees.
#[derive(Clone)]
pub enum Stage {
    /// Simple transformation function
    Transform {
        /// Stage name for debugging and error messages
        name: &'static str,

        /// Transformation function
        func: TransformFn,
    },

    /// Type-based dispatch to sub-pipelines
    Dispatch {
        /// Stage name for debugging and error messages
        name: &'static str,

        /// Extract type key from current data
        type_fn: TypeKeyFn,

        /// Map from type key to sub-pipeline
        branches: HashMap<String, Pipeline>,

        /// Default branch for unknown type keys (required)
        default: Pipeline,
    },
}

impl Stage {
    /// Get stage name for debugging
    pub fn name(&self) -> &'static str {
        match self {
            Stage::Transform { name, .. } => name,
            Stage::Dispatch { name, .. } => name,
        }
    }
}

/// Pipeline - a sequence of stages
///
/// Pipelines transform data through multiple stages, with each stage
/// producing input for the next stage.
///
/// # Examples
///
/// Simple chain:
/// ```ignore
/// Pipeline {
///     stages: vec![
///         Stage::Transform { name: "parse", func: parse_content },
///         Stage::Transform { name: "build", func: build_ast },
///     ]
/// }
/// ```
///
/// With dispatch:
/// ```ignore
/// Pipeline {
///     stages: vec![
///         Stage::Transform { name: "classify", func: classify_type },
///         Stage::Dispatch {
///             name: "process",
///             type_fn: extract_type,
///             branches: hashmap! {
///                 "Citation" => Pipeline { stages: [...] },
///                 "Footnote" => Pipeline { stages: [...] },
///             },
///             default: Pipeline { stages: [fallback] },
///         },
///     ]
/// }
/// ```
#[derive(Clone)]
pub struct Pipeline {
    /// Sequence of stages to execute
    pub stages: Vec<Stage>,
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Create pipeline from a vector of stages
    pub fn from_stages(stages: Vec<Stage>) -> Self {
        Self { stages }
    }

    /// Check if pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Get number of stages
    pub fn len(&self) -> usize {
        self.stages.len()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing pipelines with fluent API
///
/// # Examples
///
/// ```ignore
/// let pipeline = PipelineBuilder::new()
///     .then("parse", parse_content)
///     .then("classify", classify_type)
///     .dispatch(
///         "process",
///         extract_type,
///         hashmap! {
///             "Citation" => Pipeline::from_stages(vec![...]),
///         },
///         Pipeline::from_stages(vec![fallback_stage])
///     )
///     .build();
/// ```
pub struct PipelineBuilder {
    stages: Vec<Stage>,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a transform stage to the pipeline
    pub fn then(mut self, name: &'static str, func: TransformFn) -> Self {
        self.stages.push(Stage::Transform { name, func });
        self
    }

    /// Add a dispatch stage to the pipeline
    pub fn dispatch(
        mut self,
        name: &'static str,
        type_fn: TypeKeyFn,
        branches: HashMap<String, Pipeline>,
        default: Pipeline,
    ) -> Self {
        self.stages.push(Stage::Dispatch {
            name,
            type_fn,
            branches,
            default,
        });
        self
    }

    /// Build the final pipeline
    pub fn build(self) -> Pipeline {
        Pipeline {
            stages: self.stages,
        }
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, ScannerToken, SourceSpan};
    use crate::semantic::elements::inlines::engine::pipeline_data::MatchedSpan;

    #[allow(dead_code)]
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

    fn identity_transform(data: StageData) -> StageResult {
        Ok(data)
    }

    fn uppercase_transform(data: StageData) -> StageResult {
        // This is a test transform that doesn't actually change the data
        // In real usage, transforms would create new pipeline data
        Ok(data)
    }

    fn extract_inline_name(data: &StageData) -> String {
        if let Ok(span) = data.downcast::<MatchedSpan>() {
            span.inline_name.clone()
        } else {
            "unknown".to_string()
        }
    }

    #[test]
    fn test_stage_name_access() {
        let stage1 = Stage::Transform {
            name: "test",
            func: identity_transform,
        };
        assert_eq!(stage1.name(), "test");

        let stage2 = Stage::Dispatch {
            name: "dispatch",
            type_fn: extract_inline_name,
            branches: HashMap::new(),
            default: Pipeline::new(),
        };
        assert_eq!(stage2.name(), "dispatch");
    }

    #[test]
    fn test_pipeline_creation() {
        let pipeline = Pipeline::new();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.len(), 0);

        let pipeline = Pipeline::from_stages(vec![Stage::Transform {
            name: "test",
            func: identity_transform,
        }]);
        assert!(!pipeline.is_empty());
        assert_eq!(pipeline.len(), 1);
    }

    #[test]
    fn test_pipeline_builder_simple_chain() {
        let pipeline = PipelineBuilder::new()
            .then("first", identity_transform)
            .then("second", uppercase_transform)
            .build();

        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline.stages[0].name(), "first");
        assert_eq!(pipeline.stages[1].name(), "second");
    }

    #[test]
    fn test_pipeline_builder_with_dispatch() {
        let citation_pipeline = Pipeline::from_stages(vec![Stage::Transform {
            name: "parse_citation",
            func: identity_transform,
        }]);

        let default_pipeline = Pipeline::from_stages(vec![Stage::Transform {
            name: "fallback",
            func: identity_transform,
        }]);

        let mut branches = HashMap::new();
        branches.insert("Citation".to_string(), citation_pipeline);

        let pipeline = PipelineBuilder::new()
            .then("classify", identity_transform)
            .dispatch("process", extract_inline_name, branches, default_pipeline)
            .build();

        assert_eq!(pipeline.len(), 2);
        assert_eq!(pipeline.stages[0].name(), "classify");
        assert_eq!(pipeline.stages[1].name(), "process");

        // Verify dispatch stage structure
        match &pipeline.stages[1] {
            Stage::Dispatch {
                branches, default, ..
            } => {
                assert_eq!(branches.len(), 1);
                assert!(branches.contains_key("Citation"));
                assert_eq!(default.len(), 1);
            }
            _ => panic!("Expected dispatch stage"),
        }
    }

    #[test]
    fn test_pipeline_clone() {
        let pipeline = PipelineBuilder::new()
            .then("test", identity_transform)
            .build();

        let cloned = pipeline.clone();
        assert_eq!(cloned.len(), pipeline.len());
        assert_eq!(cloned.stages[0].name(), "test");
    }

    #[test]
    fn test_nested_dispatch_support() {
        // Test that we can create nested dispatch structures
        let inner_dispatch = PipelineBuilder::new()
            .dispatch(
                "inner",
                extract_inline_name,
                HashMap::new(),
                Pipeline::new(),
            )
            .build();

        let mut outer_branches = HashMap::new();
        outer_branches.insert("type1".to_string(), inner_dispatch);

        let pipeline = PipelineBuilder::new()
            .dispatch(
                "outer",
                extract_inline_name,
                outer_branches,
                Pipeline::new(),
            )
            .build();

        assert_eq!(pipeline.len(), 1);

        // Verify nested structure
        match &pipeline.stages[0] {
            Stage::Dispatch { branches, .. } => {
                let inner = branches.get("type1").unwrap();
                assert_eq!(inner.len(), 1);
                match &inner.stages[0] {
                    Stage::Dispatch { .. } => {} // Success - nested dispatch
                    _ => panic!("Expected inner dispatch stage"),
                }
            }
            _ => panic!("Expected outer dispatch stage"),
        }
    }
}
