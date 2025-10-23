//! Tests for VerbatimScanner::scan_boundaries() (Issue #132)
//!
//! Comprehensive tests for the new boundary-only scanning approach

use txxt::cst::WallType;
use txxt::syntax::verbatim_scanning::VerbatimScanner;

#[test]
fn test_scan_boundaries_simple_in_flow() {
    let input = "Code example:\n    def hello():\n        print('hi')\n:: python ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    let boundary = &boundaries[0];

    assert_eq!(boundary.title_line, 1);
    assert_eq!(boundary.terminator_line, 4);
    assert_eq!(boundary.title, "Code example");
    assert_eq!(boundary.label_raw, "python");
    assert_eq!(boundary.wall_type, WallType::InFlow(0));
    assert_eq!(boundary.title_indent, 0);
    assert_eq!(boundary.content_start, Some(2));
    assert_eq!(boundary.content_end, Some(3));
}

#[test]
fn test_scan_boundaries_stretched_mode() {
    // Use content without colons to avoid ambiguity
    // Stretched mode: content at absolute column 1 (wall position)
    let input = "Title:\n import os\n print('hello')\n:: python ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    let boundary = &boundaries[0];

    assert_eq!(boundary.title, "Title");
    assert_eq!(boundary.label_raw, "python");
    assert_eq!(boundary.wall_type, WallType::Stretched);
    assert_eq!(boundary.content_start, Some(2));
    assert_eq!(boundary.content_end, Some(3));
}

#[test]
fn test_scan_boundaries_empty_block() {
    let input = "Image:\n:: image:src=photo.png ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    let boundary = &boundaries[0];

    assert_eq!(boundary.title, "Image");
    assert_eq!(boundary.label_raw, "image:src=photo.png");
    assert_eq!(boundary.wall_type, WallType::InFlow(0));
    assert_eq!(boundary.content_start, None);
    assert_eq!(boundary.content_end, None);
}

#[test]
fn test_scan_boundaries_with_parameters() {
    let input = "Code:\n    x = 1\n:: python version=3.11,style=pep8 ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].label_raw, "python version=3.11,style=pep8");
}

#[test]
fn test_scan_boundaries_multiple_blocks() {
    let input = "First:\n    content1\n:: label1 ::\n\nSecond:\n    content2\n:: label2 ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 2);

    assert_eq!(boundaries[0].title, "First");
    assert_eq!(boundaries[0].label_raw, "label1");
    assert_eq!(boundaries[0].title_line, 1);
    assert_eq!(boundaries[0].terminator_line, 3);

    assert_eq!(boundaries[1].title, "Second");
    assert_eq!(boundaries[1].label_raw, "label2");
    assert_eq!(boundaries[1].title_line, 5);
    assert_eq!(boundaries[1].terminator_line, 7);
}

#[test]
fn test_scan_boundaries_indented_block() {
    let input = "    Indented title:\n        content\n    :: label ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    let boundary = &boundaries[0];

    assert_eq!(boundary.title, "Indented title");
    assert_eq!(boundary.title_indent, 4);
    assert_eq!(boundary.wall_type, WallType::InFlow(4));
}

#[test]
fn test_scan_boundaries_with_blank_lines_in_content() {
    let input = "Code:\n    line1\n\n    line3\n:: label ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].content_start, Some(2));
    assert_eq!(boundaries[0].content_end, Some(4));
}

#[test]
fn test_scan_boundaries_no_verbatim() {
    let input = "Just regular text\nNo verbatim here";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 0);
}

#[test]
fn test_scan_boundaries_nested_indentation() {
    let input = "Outer:\n    def foo():\n        nested\n            deep\n:: code ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].content_start, Some(2));
    assert_eq!(boundaries[0].content_end, Some(4));
}

#[test]
fn test_scan_boundaries_title_with_spaces() {
    let input = "Title with many words:\n    content\n:: label ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].title, "Title with many words");
}

#[test]
fn test_scan_boundaries_label_with_dots() {
    let input = "Code:\n    x\n:: org.example.custom ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].label_raw, "org.example.custom");
}

#[test]
fn test_is_verbatim_content_boundary() {
    let input = "Code:\n    line1\n    line2\n:: label ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    // Line 1 is title, not content
    assert!(!scanner.is_verbatim_content_boundary(1, &boundaries));

    // Lines 2-3 are content
    assert!(scanner.is_verbatim_content_boundary(2, &boundaries));
    assert!(scanner.is_verbatim_content_boundary(3, &boundaries));

    // Line 4 is terminator, not content
    assert!(!scanner.is_verbatim_content_boundary(4, &boundaries));
}

#[test]
fn test_is_verbatim_content_boundary_empty_block() {
    let input = "Empty:\n:: label ::";
    let scanner = VerbatimScanner::new();
    let boundaries = scanner.scan_boundaries(input);

    // Empty block has no content lines
    assert!(!scanner.is_verbatim_content_boundary(1, &boundaries));
    assert!(!scanner.is_verbatim_content_boundary(2, &boundaries));
}

#[test]
fn test_scan_boundaries_comparison_with_old_scan() {
    // Verify scan_boundaries produces equivalent results to old scan()
    let input = "Example:\n    code\n:: label ::";
    let scanner = VerbatimScanner::new();

    let boundaries = scanner.scan_boundaries(input);
    let blocks = scanner.scan(input);

    assert_eq!(boundaries.len(), blocks.len());

    if !boundaries.is_empty() {
        let boundary = &boundaries[0];
        let block = &blocks[0];

        assert_eq!(boundary.title_line, block.block_start);
        assert_eq!(boundary.terminator_line, block.block_end);
        assert_eq!(boundary.content_start, block.content_start);
        assert_eq!(boundary.content_end, block.content_end);
    }
}
