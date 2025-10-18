//! Unist Specification Compatibility
//!
//! This module provides full compatibility with the [Unist specification](https://github.com/syntax-tree/unist),
//! enabling TXXT AST nodes to interoperate with the broader syntax-tree ecosystem.
//!
//! # What is Unist?
//!
//! Unist (Universal Syntax Tree) is a specification for syntax trees that:
//! - Defines a minimal, extensible interface for tree structures
//! - Is implemented by mdast (Markdown), hast (HTML), nlcst (natural language), xast (XML)
//! - Has 30M+ monthly downloads of utilities in JavaScript
//! - Provides well-defined traversal and transformation patterns
//!
//! # Why Unist for TXXT?
//!
//! ## Perfect Alignment
//!
//! 1. **Text Format Focus**: Unist is designed for text/markup formats like TXXT
//! 2. **Lossless Trees**: Both preserve all source information including whitespace
//! 3. **Position Tracking**: Character-level precision for language servers
//! 4. **Extensibility**: Unist explicitly designed to be extended
//!
//! ## Practical Benefits
//!
//! 1. **Format Conversion**: Bridge to Markdown, HTML, and other formats
//! 2. **Standard Semantics**: Well-defined parent/child/sibling relationships
//! 3. **Proven Patterns**: Battle-tested traversal and transformation patterns
//! 4. **Ecosystem Tools**: Leverage existing utilities and conventions
//!
//! ## Rust vs JavaScript
//!
//! While Unist has primarily JavaScript implementations, the *specification* is
//! language-agnostic. We benefit from:
//! - Standardized semantics (what is a parent, child, sibling, position)
//! - Conversion patterns (how to transform between formats)
//! - Design principles (lossless, extensible, well-typed)
//!
//! We DON'T need:
//! - JavaScript interop (no FFI, no JSON serialization overhead)
//! - JS utilities (we implement in idiomatic Rust)
//! - Runtime compatibility (compile-time adapter layer)
//!
//! # Unist Core Interfaces
//!
//! ## Node (All Nodes)
//!
//! ```webidl
//! interface Node {
//!   type: string
//!   data: Data?
//!   position: Position?
//! }
//! ```
//!
//! Every TXXT AST node implements the Unist Node interface through [`UnistNode`].
//!
//! ## Parent (Container Nodes)
//!
//! ```webidl
//! interface Parent <: Node {
//!   children: [Node]
//! }
//! ```
//!
//! TXXT container nodes (List, Session, Container, etc.) implement [`UnistParent`].
//!
//! ## Literal (Leaf Nodes)
//!
//! ```webidl
//! interface Literal <: Node {
//!   value: any
//! }
//! ```
//!
//! TXXT text nodes implement [`UnistLiteral`].
//!
//! # Position Tracking
//!
//! Unist defines precise position information for language server features:
//!
//! ```webidl
//! interface Position {
//!   start: Point
//!   end: Point
//! }
//!
//! interface Point {
//!   line: number >= 1      // 1-indexed line number
//!   column: number >= 1    // 1-indexed column number
//!   offset: number >= 0?   // 0-indexed character offset
//! }
//! ```
//!
//! TXXT's `ScannerTokenSequence` maps directly to Unist's `Position`.
//!
//! # Usage Examples
//!
//! ```rust
//! use txxt::ast::query::unist::UnistNode;
//! use txxt::ast::nodes::paragraph::Paragraph;
//!
//! # fn example(para: &Paragraph) {
//! // Access Unist interface
//! let node_type = para.unist_type();
//! let position = para.unist_position();
//!
//! // Check node category
//! if para.is_parent() {
//!     let children = para.unist_children();
//!     println!("Has {} children", children.len());
//! }
//!
//! if para.is_literal() {
//!     let value = para.unist_value();
//!     println!("Text: {}", value);
//! }
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unist Point - A position in a source file
///
/// Represents a single location in the source document with 1-indexed line/column
/// and optional 0-indexed offset.
///
/// # Unist Specification
///
/// From [Unist Point spec](https://github.com/syntax-tree/unist#point):
/// - `line`: 1-indexed line number in source file
/// - `column`: 1-indexed column number in source file  
/// - `offset`: Optional 0-indexed character offset from start of file
///
/// # Examples
///
/// ```rust
/// use txxt::ast::query::unist::Point;
///
/// // Beginning of file
/// let start = Point::new(1, 1, Some(0));
///
/// // After "hello" on first line
/// let after_hello = Point::new(1, 6, Some(5));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    /// 1-indexed line number
    pub line: usize,
    /// 1-indexed column number
    pub column: usize,
    /// Optional 0-indexed character offset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

impl Point {
    /// Create a new point with line, column, and optional offset
    pub fn new(line: usize, column: usize, offset: Option<usize>) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Create a point without offset information
    pub fn without_offset(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            offset: None,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(offset) = self.offset {
            write!(f, "{}:{} ({})", self.line, self.column, offset)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

/// Unist Position - Location of a node in source
///
/// Represents the span of a node in the source document with start and end points.
/// Nodes without position information are "generated" (not present in source).
///
/// # Unist Specification
///
/// From [Unist Position spec](https://github.com/syntax-tree/unist#position):
/// - `start`: Location of first character of parsed region
/// - `end`: Location of first character *after* parsed region
///
/// # Examples
///
/// ```rust
/// use txxt::ast::query::unist::{Position, Point};
///
/// // Position of "hello" at start of file
/// let pos = Position::new(
///     Point::new(1, 1, Some(0)),
///     Point::new(1, 6, Some(5))
/// );
///
/// assert_eq!(pos.start.line, 1);
/// assert_eq!(pos.end.column, 6);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Start point of the node
    pub start: Point,
    /// End point of the node (first character after the node)
    pub end: Point,
}

impl Position {
    /// Create a new position from start and end points
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// Check if this position contains a given point
    pub fn contains(&self, point: &Point) -> bool {
        // If we have offsets, use them for precise comparison
        if let (Some(start_off), Some(end_off), Some(point_off)) =
            (self.start.offset, self.end.offset, point.offset)
        {
            return start_off <= point_off && point_off < end_off;
        }

        // Otherwise use line/column comparison
        let after_start = point.line > self.start.line
            || (point.line == self.start.line && point.column >= self.start.column);

        let before_end = point.line < self.end.line
            || (point.line == self.end.line && point.column < self.end.column);

        after_start && before_end
    }

    /// Get the length of this position in characters (if offset available)
    pub fn length(&self) -> Option<usize> {
        match (self.start.offset, self.end.offset) {
            (Some(start), Some(end)) => Some(end.saturating_sub(start)),
            _ => None,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} → {}", self.start, self.end)
    }
}

/// Unist Node - Universal interface for all AST nodes
///
/// This trait defines the core Unist Node interface that all TXXT AST nodes implement.
/// It provides the minimal common interface: type, data, and position.
///
/// # Unist Specification
///
/// From [Unist Node spec](https://github.com/syntax-tree/unist#node):
/// ```webidl
/// interface Node {
///   type: string
///   data: Data?
///   position: Position?
/// }
/// ```
///
/// # Implementation
///
/// All TXXT AST nodes implement this trait to enable:
/// - Uniform traversal and querying
/// - Format conversion (e.g., to/from Markdown)
/// - Language server position queries
/// - Ecosystem tool integration
pub trait UnistNode {
    /// Get the Unist type string for this node
    ///
    /// This corresponds to the node type in the TXXT AST enum variants
    /// (e.g., "Paragraph", "List", "Session", "Container").
    fn unist_type(&self) -> &str;

    /// Get the position of this node in the source (if not generated)
    ///
    /// Returns `None` for generated nodes that don't exist in the source.
    fn unist_position(&self) -> Option<Position>;

    /// Check if this node is a Parent (has children)
    fn is_parent(&self) -> bool {
        false
    }

    /// Check if this node is a Literal (has value)
    fn is_literal(&self) -> bool {
        false
    }

    /// Check if this node is generated (no position information)
    fn is_generated(&self) -> bool {
        self.unist_position().is_none()
    }
}

/// Unist Parent - Nodes that contain children
///
/// Extends UnistNode for container nodes that have child nodes.
///
/// # Unist Specification
///
/// From [Unist Parent spec](https://github.com/syntax-tree/unist#parent):
/// ```webidl
/// interface Parent <: Node {
///   children: [Node]
/// }
/// ```
///
/// # TXXT Nodes
///
/// Parent nodes in TXXT include:
/// - `List` (contains ListItems)
/// - `Session` (contains nested sessions/content)
/// - `Container` (contains nested blocks)
/// - `ListItem` (contains item content)
/// - `Definition` (contains definition content)
pub trait UnistParent: UnistNode {
    /// Get the children of this node as Unist nodes
    ///
    /// Returns a vector of child node references that implement UnistNode.
    fn unist_children(&self) -> Vec<&dyn UnistNode>;

    /// Get the number of children
    fn child_count(&self) -> usize {
        self.unist_children().len()
    }

    /// Check if this node has any children
    fn is_empty(&self) -> bool {
        self.child_count() == 0
    }

    /// Get the head (first child) if it exists
    fn head(&self) -> Option<&dyn UnistNode> {
        self.unist_children().first().copied()
    }

    /// Get the tail (last child) if it exists
    fn tail(&self) -> Option<&dyn UnistNode> {
        self.unist_children().last().copied()
    }
}

/// Unist Literal - Nodes that contain a value
///
/// Extends UnistNode for leaf nodes that contain text or data values.
///
/// # Unist Specification
///
/// From [Unist Literal spec](https://github.com/syntax-tree/unist#literal):
/// ```webidl
/// interface Literal <: Node {
///   value: any
/// }
/// ```
///
/// # TXXT Nodes
///
/// Literal nodes in TXXT include:
/// - Text content (inline text runs)
/// - Code spans (verbatim inline code)
/// - Math expressions (inline/block math)
pub trait UnistLiteral: UnistNode {
    /// Get the value of this literal node as a string
    ///
    /// For text nodes, this is the text content.
    /// For other literals, this is a string representation.
    fn unist_value(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(5, 10, Some(42));
        assert_eq!(point.line, 5);
        assert_eq!(point.column, 10);
        assert_eq!(point.offset, Some(42));
    }

    #[test]
    fn test_point_without_offset() {
        let point = Point::without_offset(3, 7);
        assert_eq!(point.line, 3);
        assert_eq!(point.column, 7);
        assert_eq!(point.offset, None);
    }

    #[test]
    fn test_point_display() {
        let with_offset = Point::new(2, 5, Some(15));
        assert_eq!(with_offset.to_string(), "2:5 (15)");

        let without_offset = Point::without_offset(2, 5);
        assert_eq!(without_offset.to_string(), "2:5");
    }

    #[test]
    fn test_position_contains() {
        let pos = Position::new(
            Point::new(1, 1, Some(0)),
            Point::new(1, 10, Some(9)),
        );

        // Inside the range
        assert!(pos.contains(&Point::new(1, 5, Some(4))));

        // At start (inclusive)
        assert!(pos.contains(&Point::new(1, 1, Some(0))));

        // At end (exclusive)
        assert!(!pos.contains(&Point::new(1, 10, Some(9))));

        // Before start
        assert!(!pos.contains(&Point::new(1, 1, None)));

        // After end
        assert!(!pos.contains(&Point::new(2, 1, Some(10))));
    }

    #[test]
    fn test_position_length() {
        let pos = Position::new(
            Point::new(1, 1, Some(0)),
            Point::new(1, 6, Some(5)),
        );
        assert_eq!(pos.length(), Some(5));

        let no_offset = Position::new(
            Point::without_offset(1, 1),
            Point::without_offset(1, 6),
        );
        assert_eq!(no_offset.length(), None);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(
            Point::new(1, 1, Some(0)),
            Point::new(2, 5, Some(25)),
        );
        assert_eq!(pos.to_string(), "1:1 (0) → 2:5 (25)");
    }

    // Mock implementation for testing traits
    struct MockNode {
        type_name: String,
        position: Option<Position>,
    }

    impl UnistNode for MockNode {
        fn unist_type(&self) -> &str {
            &self.type_name
        }

        fn unist_position(&self) -> Option<Position> {
            self.position
        }
    }

    #[test]
    fn test_unist_node_generated() {
        let generated = MockNode {
            type_name: "Test".to_string(),
            position: None,
        };
        assert!(generated.is_generated());

        let with_pos = MockNode {
            type_name: "Test".to_string(),
            position: Some(Position::new(
                Point::new(1, 1, Some(0)),
                Point::new(1, 5, Some(4)),
            )),
        };
        assert!(!with_pos.is_generated());
    }
}
