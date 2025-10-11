use crate::ast::AstNode;

/// Utilities for processing and grouping list items
pub struct ListProcessor;

impl ListProcessor {
    /// Group consecutive list items into list elements
    ///
    /// Takes a flat sequence of elements and groups consecutive list_item elements
    /// into list containers. This implements flat list parsing.
    pub fn group_list_items(elements: Vec<AstNode>) -> Vec<AstNode> {
        let mut result = Vec::new();
        let mut current_list: Option<AstNode> = None;

        for element in elements {
            if element.node_type == "list_item" {
                // This is a list item - add it to current list or start a new one
                if let Some(ref mut list) = current_list {
                    // Add to existing list
                    list.add_child(element);
                } else {
                    // Start a new list
                    let mut new_list = AstNode::new("list".to_string());

                    // Set location based on first list item
                    if let (Some(start), Some(end)) = (element.start_line, element.end_line) {
                        new_list.set_location(start, end);
                    }

                    new_list.add_child(element);
                    current_list = Some(new_list);
                }
            } else {
                // Not a list item - finish current list if any, then add this element
                if let Some(list) = current_list.take() {
                    result.push(list);
                }
                result.push(element);
            }
        }

        // Don't forget the final list if we ended with list items
        if let Some(list) = current_list {
            result.push(list);
        }

        result
    }
}
