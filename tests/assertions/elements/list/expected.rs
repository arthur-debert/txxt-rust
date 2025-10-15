//! List element expected value structs for assertions.

use txxt::ast::elements::list::NumberingStyle;

/// Expected properties for list validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during Parser 2.1.2
pub struct ListExpected<'a> {
    /// List decoration style (Plain, Numerical, Alphabetical, Roman)
    pub style: Option<NumberingStyle>,

    /// Number of list items
    pub item_count: Option<usize>,

    /// Text content of each item (in order)
    pub item_text: Option<Vec<&'a str>>,

    /// Which items have nested containers (in order)
    pub has_nested: Option<Vec<bool>>,

    /// Number of annotations attached to list
    pub annotation_count: Option<usize>,

    /// Has annotation with this label
    pub has_annotation: Option<&'a str>,
}
