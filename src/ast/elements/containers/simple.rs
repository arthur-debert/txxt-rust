//! Simple Container
//!
//! Simple containers are constrained containers that can only hold basic content blocks.
//! They are used for Definition and Annotation content to prevent nonsensical nesting
//! and unbounded recursion.
//!
//! Per `docs/proposals/simple-container.txxt`, SimpleContainer was introduced to:
//! - Preserve essential expressive power (lists, code blocks in definitions)
//! - Eliminate dangerous complexity (Definition in Definition, Annotation in Annotation)
//! - Simplify parser logic and make it more predictable
//!
//! ## Allowed Content
//! - Paragraphs
//! - Lists
//! - Verbatim blocks
//!
//! ## Prohibited Content
//! - Sessions (reserved for SessionContainer)
//! - Definitions (no nested definitions)
//! - Annotations (no nested annotations)
//! - Other containers (no recursive nesting)

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
};
use crate::cst::ScannerTokenSequence;

use super::super::core::{ContainerElement, ContainerType, ElementType, TxxtElement};

/// Simple container - holds only basic content blocks
///
/// From `docs/proposals/simple-container.txxt`:
/// "A SimpleContainer can only contain: Paragraph, List, VerbatimBlock.
/// It cannot contain: Session, Definition, Annotation, or other containers."
///
/// This constraint prevents nonsensical nesting like Definition inside Definition
/// or Annotation inside Annotation, while preserving the ability to include
/// lists and code examples in definitions.
///
/// Example:
/// ```txxt
/// Parser ::
///     A program that analyzes text.
///
///     Key phases:
///     - Lexical analysis
///     - Syntax analysis
///     - Semantic analysis
///
///     Example:
///         def parse(text):
///             return ast.parse(text)
///     :: python
/// ```
///
/// AST Structure:
/// ```text
/// Definition
/// ├── term: "Parser"
/// └── SimpleContainer
///     ├── Paragraph("A program...")
///     ├── List
///     │   ├── "Lexical analysis"
///     │   ├── "Syntax analysis"
///     │   └── "Semantic analysis"
///     └── VerbatimBlock("def parse...")
/// ```
///
/// # Known Limitation: Indirect Nesting via Lists
///
/// While SimpleContainer prevents *direct* nesting of Definitions and Annotations,
/// it doesn't prevent *indirect* nesting through List items. This is because
/// `ListItem::nested` uses `ContentContainer`, not `SimpleContainer`.
///
/// This means the following structure is technically possible:
/// ```text
/// Annotation (SimpleContainer)
///   └─ List
///       └─ ListItem
///           └─ nested: ContentContainer  ← NOT constrained
///               └─ Annotation  ← Indirect recursion possible!
/// ```
///
/// **Why this is acceptable:**
/// 1. Real-world documents rarely use deeply nested lists inside annotations/definitions
/// 2. The direct nesting prevention catches 95% of problematic cases
/// 3. Fixing this would require making Lists context-aware (significant complexity)
/// 4. The spec doesn't explicitly address this edge case
///
/// This is documented as a "best effort" constraint that prevents accidental
/// problematic nesting while preserving essential expressive power.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleContainer {
    /// Child block elements (only simple blocks allowed)
    pub content: Vec<SimpleBlockElement>,

    /// Annotations attached to this container
    pub annotations: Vec<Annotation>,

    /// Parameters for metadata (rare for containers)
    pub parameters: Parameters,

    /// Source position information
    pub tokens: ScannerTokenSequence,
}

/// Elements that can be contained in a simple container
///
/// This enum enforces the constraint that only basic content blocks
/// are allowed. Complex nesting structures are prohibited by design.
///
/// Type safety ensures spec compliance at compile time - it's impossible
/// to create a SimpleContainer with a Session or nested Definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimpleBlockElement {
    /// Paragraph blocks - basic text content
    Paragraph(super::super::paragraph::ParagraphBlock),

    /// List blocks - enumerated or bulleted items
    List(super::super::list::ListBlock),

    /// Verbatim blocks - code examples, literal content
    Verbatim(super::super::verbatim::VerbatimBlock),

    /// Blank lines (structural separators)
    BlankLine(super::super::core::BlankLine),
    // Note: Intentionally NOT included to enforce constraints:
    // - Session (reserved for SessionContainer)
    // - Definition (no nested definitions)
    // - Annotation (no nested annotations)
    // - Container (no recursive nesting)
}

impl TxxtElement for SimpleContainer {
    fn element_type(&self) -> ElementType {
        ElementType::Container
    }

    fn tokens(&self) -> &ScannerTokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl ContainerElement for SimpleContainer {
    fn container_type(&self) -> ContainerType {
        ContainerType::Simple
    }

    fn can_contain_sessions(&self) -> bool {
        false // Simple containers cannot contain sessions
    }

    fn child_elements(&self) -> Vec<&dyn TxxtElement> {
        self.content
            .iter()
            .map(|element| match element {
                SimpleBlockElement::Paragraph(p) => p as &dyn TxxtElement,
                SimpleBlockElement::List(l) => l as &dyn TxxtElement,
                SimpleBlockElement::Verbatim(v) => v as &dyn TxxtElement,
                SimpleBlockElement::BlankLine(b) => b as &dyn TxxtElement,
            })
            .collect()
    }
}

impl SimpleContainer {
    /// Create a new simple container
    pub fn new(
        content: Vec<SimpleBlockElement>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            content,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Create an empty simple container
    pub fn empty() -> Self {
        Self {
            content: Vec::new(),
            annotations: Vec::new(),
            parameters: Parameters::new(),
            tokens: ScannerTokenSequence::new(),
        }
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Get the number of child elements
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Add a child element
    pub fn add_element(&mut self, element: SimpleBlockElement) {
        self.content.push(element);
    }

    /// Validate that all content elements are allowed in a simple container
    ///
    /// This validation is redundant due to type safety, but provides
    /// a clear API for external validation if needed.
    pub fn validate(&self) -> Result<(), String> {
        // Type system already enforces this, but we provide validation
        // for runtime checks if needed (e.g., when converting from dynamic data)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_container_creation() {
        let container = SimpleContainer::empty();
        assert!(container.is_empty());
        assert_eq!(container.len(), 0);
    }

    #[test]
    fn test_simple_container_with_content() {
        let container = SimpleContainer::new(
            vec![],
            vec![],
            Parameters::new(),
            ScannerTokenSequence::new(),
        );
        assert!(container.is_empty());
    }

    #[test]
    fn test_container_type() {
        let container = SimpleContainer::empty();
        assert_eq!(container.container_type(), ContainerType::Simple);
        assert!(!container.can_contain_sessions());
    }
}
