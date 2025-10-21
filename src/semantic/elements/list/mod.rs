//! List Element Construction
//!
//! Converts high-level tokens into list AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/list/`
//! - **AST Node**: `src/ast/elements/list/block.rs`

use crate::ast::elements::containers::ContentContainer;
use crate::ast::elements::inlines::TextTransform;
use crate::ast::elements::list::block::{
    ListBlock, ListDecorationType, ListItem, NumberingForm, NumberingStyle,
};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::ast_construction::AstNode;
use crate::semantic::BlockParseError;

pub fn create_list_element_with_nesting(
    list_items_data: &[(HighLevelToken, Vec<AstNode>)],
) -> Result<ListBlock, BlockParseError> {
    let mut items = Vec::new();
    for (item_token, nested_nodes) in list_items_data {
        if let HighLevelToken::SequenceTextLine {
            marker: marker_token,
            content: content_token,
            ..
        } = item_token
        {
            let marker = match marker_token.as_ref() {
                HighLevelToken::SequenceMarker { marker, .. } => marker.clone(),
                _ => "".to_string(),
            };
            let (content, content_tokens) = match content_token.as_ref() {
                HighLevelToken::TextSpan { content, tokens, .. } => (content.clone(), tokens.clone()),
                _ => ("".to_string(), None),
            };
            let content_transforms = if !content.is_empty() {
                vec![TextTransform::Identity(
                    crate::ast::elements::inlines::Text::simple_with_tokens(&content, content_tokens),
                )]
            } else {
                vec![]
            };

            let nested_container = if !nested_nodes.is_empty() {
                let mut content_elements = Vec::new();
                for node in nested_nodes {
                    let element_node = node.to_element_node();
                    match element_node.try_into() {
                        Ok(container_element) => content_elements.push(container_element),
                        Err(e) => {
                            return Err(BlockParseError::InvalidStructure(format!(
                                "Failed to convert nested element in list item: {}",
                                e
                            )));
                        }
                    }
                }
                Some(ContentContainer::new(
                    content_elements,
                    vec![],
                    Default::default(),
                    Default::default(),
                ))
            } else {
                None
            };

            items.push(ListItem {
                marker,
                content: content_transforms,
                nested: nested_container,
                annotations: vec![],
                parameters: Default::default(),
                tokens: Default::default(),
            });
        }
    }
    let decoration_type = if items.is_empty() {
        Default::default()
    } else {
        determine_decoration_type(&items[0].marker)
    };
    Ok(ListBlock {
        decoration_type,
        items,
        annotations: vec![],
        parameters: Default::default(),
        tokens: Default::default(),
    })
}

/// Create a list element from parsed components
///
/// Lists are 2+ consecutive SequenceTextLine tokens with no blank lines between them.
///
/// # Arguments
/// * `item_tokens` - Vector of SequenceTextLine tokens
///
/// # Returns
/// * `Result<ListBlock, BlockParseError>`
pub fn create_list_element(item_tokens: &[HighLevelToken]) -> Result<ListBlock, BlockParseError> {
    if item_tokens.len() < 2 {
        return Err(BlockParseError::InvalidStructure(
            "Lists require at least 2 items".to_string(),
        ));
    }

    // Parse each item token into a ListItem
    let mut items = Vec::new();

    for token in item_tokens {
        if let HighLevelToken::SequenceTextLine {
            marker: marker_token,
            content: content_token,
            ..
        } = token
        {
            // Extract marker text from marker token
            let marker = match marker_token.as_ref() {
                HighLevelToken::SequenceMarker { marker, .. } => marker.clone(),
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "- ".to_string(), // fallback
            };

            // Extract item content text and tokens from content token
            let (item_content, content_tokens) = match content_token.as_ref() {
                HighLevelToken::TextSpan { content, tokens, .. } => (content.clone(), tokens.clone()),
                _ => (String::new(), None),
            };

            // Create TextTransform for item content with preserved source tokens
            let content_transforms = if !item_content.is_empty() {
                let text = crate::ast::elements::inlines::Text::simple_with_tokens(&item_content, content_tokens);
                vec![TextTransform::Identity(text)]
            } else {
                vec![]
            };

            items.push(ListItem {
                marker,
                // FIXME: post-parser - Parse inline formatting in content instead of using Text::simple
                content: content_transforms,
                // FIXME: post-parser - Parse nested lists instead of None
                nested: None,
                // FIXME: post-parser - Parse item-level annotations
                annotations: Vec::new(),
                // FIXME: post-parser - Extract parameters from list item
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                tokens: ScannerTokenSequence::new(),
            });
        }
    }

    // Determine decoration type from first item
    let decoration_type = determine_decoration_type(&items[0].marker);

    Ok(ListBlock {
        decoration_type,
        items,
        // FIXME: post-parser - Parse list-level annotations
        annotations: Vec::new(),
        // FIXME: post-parser - Extract parameters from list
        parameters: crate::ast::elements::components::parameters::Parameters::new(),
        tokens: ScannerTokenSequence::new(),
    })
}

/// Determine list decoration type from the first marker
fn determine_decoration_type(marker: &str) -> ListDecorationType {
    let style = if marker.starts_with('-') {
        NumberingStyle::Plain
    } else if marker.chars().next().is_some_and(|c| c.is_numeric()) {
        NumberingStyle::Numerical
    } else if marker.chars().next().is_some_and(|c| c.is_alphabetic()) {
        NumberingStyle::Alphabetical
    } else {
        NumberingStyle::Plain // fallback
    };

    ListDecorationType {
        style,
        form: NumberingForm::Short,
    }
}
