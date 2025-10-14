//! Predicate system for filtering nodes during traversal

use super::node_ref::{AstNode, AstNodeRef, NodeType};

/// Predicates for filtering nodes during traversal
///
/// Predicates are composable building blocks that can be combined to create
/// complex queries. Each predicate is a boolean test on a node.
pub enum NodePredicate {
    /// Match by node type
    Type(NodeType),

    /// Match if text content contains substring (case-insensitive)
    TextContains(String),

    /// Match if text content matches regex
    TextMatches(String),

    /// Match if node has specific parameter
    HasParameter { key: String, value: Option<String> },

    /// Match if node has any annotations
    HasAnnotations,

    /// Match if node has annotation with specific label
    AnnotationLabel(String),

    /// Match if node has children
    HasChildren,

    /// Match if node is a leaf
    IsLeaf,

    /// Match if node is at specific level
    AtLevel(usize),

    /// Custom predicate function
    Custom(Box<dyn Fn(AstNodeRef<'_>) -> bool>),

    /// Logical AND of multiple predicates
    And(Vec<NodePredicate>),

    /// Logical OR of multiple predicates
    Or(Vec<NodePredicate>),

    /// Logical NOT
    Not(Box<NodePredicate>),
}

impl NodePredicate {
    /// Evaluate predicate against a node
    pub fn matches<N: AstNode>(&self, node: &N) -> bool {
        match self {
            Self::Type(t) => node.node_type() == *t,

            Self::TextContains(text) => node
                .text_content()
                .map(|content| content.to_lowercase().contains(&text.to_lowercase()))
                .unwrap_or(false),

            Self::TextMatches(pattern) => {
                // TODO: Add regex dependency and implement
                // For now, fallback to contains
                node.text_content()
                    .map(|content| content.contains(pattern))
                    .unwrap_or(false)
            }

            Self::HasParameter { key, value } => node
                .parameters()
                .and_then(|params| params.get(key))
                .map(|param_value| {
                    value
                        .as_ref()
                        .map(|v| param_value == v)
                        .unwrap_or(true)
                })
                .unwrap_or(false),

            Self::HasAnnotations => !node.annotations().is_empty(),

            Self::AnnotationLabel(label) => node
                .annotations()
                .iter()
                .any(|ann| ann.label.as_deref() == Some(label)),

            Self::HasChildren => !node.children().is_empty(),

            Self::IsLeaf => node.children().is_empty(),

            Self::AtLevel(level) => node.level() == *level,

            Self::Custom(f) => {
                // Convert node to AstNodeRef for custom predicates
                // This is a bit awkward - might need to refactor
                // For now, just return false
                // TODO: Improve custom predicate handling
                false
            }

            Self::And(predicates) => predicates.iter().all(|p| p.matches(node)),

            Self::Or(predicates) => predicates.iter().any(|p| p.matches(node)),

            Self::Not(predicate) => !predicate.matches(node),
        }
    }

    /// Combine this predicate with another using AND
    pub fn and(self, other: NodePredicate) -> NodePredicate {
        match (self, other) {
            (NodePredicate::And(mut preds1), NodePredicate::And(preds2)) => {
                preds1.extend(preds2);
                NodePredicate::And(preds1)
            }
            (NodePredicate::And(mut preds), other) | (other, NodePredicate::And(mut preds)) => {
                preds.push(other);
                NodePredicate::And(preds)
            }
            (pred1, pred2) => NodePredicate::And(vec![pred1, pred2]),
        }
    }

    /// Combine this predicate with another using OR
    pub fn or(self, other: NodePredicate) -> NodePredicate {
        match (self, other) {
            (NodePredicate::Or(mut preds1), NodePredicate::Or(preds2)) => {
                preds1.extend(preds2);
                NodePredicate::Or(preds1)
            }
            (NodePredicate::Or(mut preds), other) | (other, NodePredicate::Or(mut preds)) => {
                preds.push(other);
                NodePredicate::Or(preds)
            }
            (pred1, pred2) => NodePredicate::Or(vec![pred1, pred2]),
        }
    }

    /// Negate this predicate
    pub fn not(self) -> NodePredicate {
        NodePredicate::Not(Box::new(self))
    }
}

// Implement Clone manually since Custom contains a closure
impl Clone for NodePredicate {
    fn clone(&self) -> Self {
        match self {
            Self::Type(t) => Self::Type(*t),
            Self::TextContains(s) => Self::TextContains(s.clone()),
            Self::TextMatches(s) => Self::TextMatches(s.clone()),
            Self::HasParameter { key, value } => Self::HasParameter {
                key: key.clone(),
                value: value.clone(),
            },
            Self::HasAnnotations => Self::HasAnnotations,
            Self::AnnotationLabel(s) => Self::AnnotationLabel(s.clone()),
            Self::HasChildren => Self::HasChildren,
            Self::IsLeaf => Self::IsLeaf,
            Self::AtLevel(l) => Self::AtLevel(*l),
            Self::Custom(_) => {
                // Can't clone closures - create a dummy that always returns false
                // This is a limitation - users should avoid cloning queries with Custom
                Self::Custom(Box::new(|_| false))
            }
            Self::And(preds) => Self::And(preds.clone()),
            Self::Or(preds) => Self::Or(preds.clone()),
            Self::Not(pred) => Self::Not(pred.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predicate_and() {
        let pred1 = NodePredicate::Type(NodeType::Paragraph);
        let pred2 = NodePredicate::HasAnnotations;

        let combined = pred1.and(pred2);

        match combined {
            NodePredicate::And(preds) => assert_eq!(preds.len(), 2),
            _ => panic!("Expected And predicate"),
        }
    }

    #[test]
    fn test_predicate_or() {
        let pred1 = NodePredicate::Type(NodeType::Paragraph);
        let pred2 = NodePredicate::Type(NodeType::Session);

        let combined = pred1.or(pred2);

        match combined {
            NodePredicate::Or(preds) => assert_eq!(preds.len(), 2),
            _ => panic!("Expected Or predicate"),
        }
    }
}
