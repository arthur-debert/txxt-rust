//! List Element Construction
//!
//! Converts high-level tokens into list AST nodes.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/list/`
//! - **AST Node**: `src/ast/elements/list/block.rs`

use crate::ast::elements::inlines::TextTransform;
use crate::ast::elements::list::block::{
    ListBlock, ListDecorationType, ListItem, NumberingForm, NumberingStyle,
};
use crate::cst::{HighLevelToken, ScannerTokenSequence};
use crate::semantic::BlockParseError;

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
        if let HighLevelToken::SequenceTextLine { content, .. } = token {
            // Extract marker and content from the SequenceTextLine
            // The content is a TextSpan with the text after the marker
            let (marker, item_content) = match content.as_ref() {
                HighLevelToken::TextSpan { content, .. } => {
                    // For now, extract marker from the beginning of content
                    // This is simplified - proper marker extraction should happen in tokenizer
                    let text = content.as_str();

                    // Simple marker detection
                    let marker = if text.starts_with("- ") {
                        "- ".to_string()
                    } else if let Some(pos) = text.find(". ") {
                        text[..=pos].to_string()
                    } else if let Some(pos) = text.find(") ") {
                        text[..=pos].to_string()
                    } else {
                        "- ".to_string() // fallback
                    };

                    let content_text = text.trim_start_matches(&marker);
                    (marker, content_text.to_string())
                }
                _ => ("- ".to_string(), String::new()),
            };

            // Create TextTransform for item content
            let content_transforms = if !item_content.is_empty() {
                let text = crate::ast::elements::inlines::Text::simple(&item_content);
                vec![TextTransform::Identity(text)]
            } else {
                vec![]
            };

            items.push(ListItem {
                marker,
                content: content_transforms,
                nested: None,
                annotations: Vec::new(),
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
        annotations: Vec::new(),
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
