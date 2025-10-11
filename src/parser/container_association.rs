use super::session_detection::SessionDetector;
use crate::ast::AstNode;
use crate::block_grouping::TokenBlock;

/// Logic for associating child blocks with parent elements and creating containers
pub struct ContainerAssociator;

impl ContainerAssociator {
    /// Associate child blocks with elements and detect sessions (for SessionContainers)
    pub fn associate_children_and_detect_sessions(
        elements: &mut [AstNode],
        parsed_child_blocks: &[(&TokenBlock, Vec<AstNode>)],
        parse_block_bottom_up: impl Fn(&TokenBlock, bool) -> Vec<AstNode>,
    ) {
        for (child_block, child_elements) in parsed_child_blocks {
            if let Some(child_start_line) = child_block.start_line {
                // Find the most recent element that could be the parent
                let mut target_element_index = None;
                let mut best_end_line = 0;
                for (i, element) in elements.iter().enumerate() {
                    if let Some(element_end_line) = element.end_line {
                        if element_end_line < child_start_line && element_end_line > best_end_line {
                            target_element_index = Some(i);
                            best_end_line = element_end_line;
                        }
                    }
                }

                if let Some(i) = target_element_index {
                    // Check if this should be a session
                    let should_be_session =
                        SessionDetector::should_be_session(&elements[i], child_elements);

                    if should_be_session {
                        // Convert to session
                        let mut session = AstNode::new("session".to_string());

                        // Copy title from original element
                        if let Some(content) = &elements[i].content {
                            session.set_attribute("title".to_string(), content.clone());
                        }

                        // Set location from original element
                        if let (Some(start), Some(end)) =
                            (elements[i].start_line, elements[i].end_line)
                        {
                            session.set_location(start, end);
                        }

                        // Create session container
                        let mut session_container = AstNode::new("session_container".to_string());
                        for child_element in child_elements {
                            session_container.add_child(child_element.clone());
                        }

                        session.add_child(session_container);
                        elements[i] = session;
                    } else if matches!(
                        elements[i].node_type.as_str(),
                        "definition" | "annotation" | "list_item"
                    ) {
                        // Create content container for non-session elements
                        // Re-parse child block with is_session_context = false to ensure no sessions
                        let content_child_elements = parse_block_bottom_up(child_block, false);
                        let mut content_container = AstNode::new("content_container".to_string());
                        for child_element in content_child_elements {
                            content_container.add_child(child_element);
                        }
                        elements[i].add_child(content_container);
                    }
                }
            }
        }
    }

    /// Associate child blocks with elements as ContentContainers (no sessions allowed)
    pub fn associate_children_as_content_containers(
        elements: &mut [AstNode],
        parsed_child_blocks: &[(&TokenBlock, Vec<AstNode>)],
    ) {
        for (child_block, child_elements) in parsed_child_blocks {
            if let Some(child_start_line) = child_block.start_line {
                // Find the most recent element that could be the parent
                let mut target_element_index = None;
                let mut best_end_line = 0;
                for (i, element) in elements.iter().enumerate() {
                    if let Some(element_end_line) = element.end_line {
                        if element_end_line < child_start_line && element_end_line > best_end_line {
                            target_element_index = Some(i);
                            best_end_line = element_end_line;
                        }
                    }
                }

                if let Some(i) = target_element_index {
                    if matches!(
                        elements[i].node_type.as_str(),
                        "definition" | "annotation" | "list_item"
                    ) {
                        // Create content container (no sessions allowed)
                        let mut content_container = AstNode::new("content_container".to_string());
                        for child_element in child_elements {
                            content_container.add_child(child_element.clone());
                        }
                        elements[i].add_child(content_container);
                    }
                }
            }
        }
    }

    /// Connect child blocks to their appropriate parent elements
    /// is_session_container determines whether we create SessionContainers or ContentContainers
    #[allow(dead_code)]
    pub fn connect_child_blocks_to_elements(
        elements: &mut [AstNode],
        child_blocks: &[TokenBlock],
        is_session_container: bool,
        parse_block_contents: impl Fn(&TokenBlock) -> Vec<AstNode>,
        parse_block_contents_as_session: impl Fn(&TokenBlock) -> Vec<AstNode>,
    ) {
        for child_block in child_blocks {
            if let Some(child_start_line) = child_block.start_line {
                // Find the element that should own this child block
                // This is the most recent element that ends just before the child block starts
                let mut target_element_index = None;
                let mut best_end_line = 0;
                for (i, element) in elements.iter().enumerate() {
                    if let Some(element_end_line) = element.end_line {
                        if element_end_line < child_start_line && element_end_line > best_end_line {
                            target_element_index = Some(i);
                            best_end_line = element_end_line;
                        }
                    }
                }

                // If we found a target element, check if it should be a session or content container
                if let Some(i) = target_element_index {
                    // Check if this might be a session (only in session containers)
                    let should_be_session = if is_session_container {
                        SessionDetector::is_session_title(&elements[i], child_block)
                    } else {
                        false
                    };

                    if should_be_session {
                        // Convert the element to a session and add a session container
                        let mut session = AstNode::new("session".to_string());

                        // Copy attributes from the original element (which becomes the session title)
                        if let Some(content) = &elements[i].content {
                            session.set_attribute("title".to_string(), content.clone());
                        }

                        // Set location from original element
                        if let (Some(start), Some(end)) =
                            (elements[i].start_line, elements[i].end_line)
                        {
                            session.set_location(start, end);
                        }

                        // Create session container for the content
                        let mut session_container = AstNode::new("session_container".to_string());

                        // Parse all elements in the child block
                        let child_elements = parse_block_contents_as_session(child_block);
                        for child_element in child_elements {
                            session_container.add_child(child_element);
                        }

                        session.add_child(session_container);
                        elements[i] = session;
                    } else if matches!(
                        elements[i].node_type.as_str(),
                        "definition" | "annotation" | "list_item"
                    ) {
                        // Create a content container for non-session elements that can have children
                        let mut content_container = AstNode::new("content_container".to_string());

                        // Parse all elements in the child block (no sessions allowed in content containers)
                        let child_elements = parse_block_contents(child_block);
                        for child_element in child_elements {
                            content_container.add_child(child_element);
                        }

                        elements[i].add_child(content_container);
                    }
                }
            }
        }
    }
}
