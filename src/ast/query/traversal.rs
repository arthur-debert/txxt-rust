//! Iterator system for AST traversal

use std::collections::HashSet;

use super::{
    node_ref::{AstNode, AstNodeRef},
    query_builder::NodeQuery,
};

/// Iterator over AST nodes matching query criteria
///
/// This iterator lazily evaluates predicates as it traverses the tree,
/// avoiding unnecessary work for nodes that don't match.
pub struct NodeIterator<'ast> {
    query: NodeQuery<'ast>,
    stack: Vec<(AstNodeRef<'ast>, usize)>, // (node, depth)
    visited: HashSet<usize>,                // For cycle detection
}

impl<'ast> NodeIterator<'ast> {
    /// Create a new iterator from a query
    pub(crate) fn new(query: NodeQuery<'ast>) -> Self {
        let root = query.root;
        Self {
            query,
            stack: vec![(root, 0)],
            visited: HashSet::new(),
        }
    }
}

impl<'ast> Iterator for NodeIterator<'ast> {
    type Item = AstNodeRef<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, depth) = self.stack.pop()?;

            // Skip if already visited (cycle detection)
            let node_id = node.id();
            if !self.visited.insert(node_id) {
                continue;
            }

            // Check depth constraints
            if let Some((min, max)) = self.query.depth_range {
                if depth < min || depth > max {
                    // Still need to traverse children if we haven't hit max depth
                    if depth < max {
                        self.add_children_to_stack(node, depth);
                    }
                    continue;
                }
            }

            // Add children to stack based on traversal mode
            self.add_children_to_stack(node, depth);

            // Check if node matches all predicates
            if self.query.predicates.iter().all(|p| p.matches(&node)) {
                return Some(node);
            }
        }
    }
}

impl<'ast> NodeIterator<'ast> {
    /// Add children to traversal stack based on traversal mode
    fn add_children_to_stack(&mut self, node: AstNodeRef<'ast>, depth: usize) {
        let children = node.children();

        match self.query.traversal_mode {
            TraversalMode::DepthFirst => {
                // Add children in reverse order for DFS (so first child is processed first)
                for child in children.into_iter().rev() {
                    self.stack.push((child, depth + 1));
                }
            }
            TraversalMode::BreadthFirst => {
                // Add children at beginning of stack for BFS
                for child in children.into_iter().rev() {
                    self.stack.insert(0, (child, depth + 1));
                }
            }
        }
    }
}

/// Traversal modes for tree traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalMode {
    /// Depth-first traversal (visit children before siblings)
    DepthFirst,
    /// Breadth-first traversal (visit siblings before children)
    BreadthFirst,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        base::Document,
        blocks::Block,
        structure::{Container, ContainerType, Paragraph},
    };

    #[test]
    fn test_iterator_basic() {
        let doc = Document::new("test".to_string());
        let query = NodeQuery::from(AstNodeRef::Document(&doc));

        let nodes: Vec<_> = query.find().collect();

        // Should at least find the document and its container
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_depth_first_traversal() {
        let doc = Document::new("test".to_string());
        let query = NodeQuery::from(AstNodeRef::Document(&doc)).depth_first();

        assert_eq!(query.traversal_mode, TraversalMode::DepthFirst);
    }

    #[test]
    fn test_breadth_first_traversal() {
        let doc = Document::new("test".to_string());
        let query = NodeQuery::from(AstNodeRef::Document(&doc)).breadth_first();

        assert_eq!(query.traversal_mode, TraversalMode::BreadthFirst);
    }
}
