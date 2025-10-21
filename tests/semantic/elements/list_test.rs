//! Tests for list element construction
//!
//! Tests that list components are correctly converted to list AST nodes.

use txxt::semantic::elements::list::create_list_element;

/// Test that list elements are created correctly with item count
#[test]
fn test_create_list_element() {
    // Create a list with 3 items
    let item_count = 3;

    // Test the element constructor directly
    let result = create_list_element(item_count);

    assert!(result.is_ok());
    let _list_block = result.unwrap();
    // TODO: Check list item count when properly implemented
}

/// Test that list elements can be created with minimum items
#[test]
fn test_create_list_element_minimum_items() {
    // Lists require at least 2 items
    let item_count = 2;

    let result = create_list_element(item_count);

    assert!(result.is_ok());
}

/// Test that list elements can be created with many items
#[test]
fn test_create_list_element_many_items() {
    // Test with a larger list
    let item_count = 10;

    let result = create_list_element(item_count);

    assert!(result.is_ok());
}
