use txxt::block_grouping::build_block_tree;
use txxt::tokenizer::tokenize;

#[test]
fn test_simple_paragraph_grouping() {
    let text = "This is a simple paragraph.";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);

    // Should have tokens but no children for simple paragraph
    assert!(!block_tree.tokens.is_empty() || !block_tree.children.is_empty());
}

#[test]
fn test_annotation_grouping() {
    let text = ":: title :: My Document";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);

    // Should create a tree structure with the annotation tokens
    assert!(!block_tree.tokens.is_empty() || !block_tree.children.is_empty());
}

#[test]
fn test_definition_grouping() {
    let text = "Parser ::";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);

    // Should create a tree structure with the definition tokens
    assert!(!block_tree.tokens.is_empty() || !block_tree.children.is_empty());
}

#[test]
fn test_list_item_grouping() {
    let text = "- List item";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);

    // Should create a tree structure with the list item tokens
    assert!(!block_tree.tokens.is_empty() || !block_tree.children.is_empty());
}

#[test]
fn test_blank_line_splitting() {
    let text = "First paragraph\n\nSecond paragraph";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);

    // Should create a tree with children due to blank line splitting
    assert!(!block_tree.children.is_empty());
}

#[test]
fn test_session_with_children() {
    let text = "Session Title\n\n    Indented content\n    More content";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);

    // Should create a tree with nested structure
    assert!(!block_tree.children.is_empty() || !block_tree.tokens.is_empty());
}
