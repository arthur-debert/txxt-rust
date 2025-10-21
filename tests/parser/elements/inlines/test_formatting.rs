use txxt::cst::{Position, SourceSpan, ScannerToken};
use txxt::semantic::elements::formatting::parse_formatting_elements;
use txxt::ast::elements::formatting::inlines::{Text, TextTransform};

fn create_bold_delimiter(start: usize, end: usize) -> ScannerToken {
    ScannerToken::BoldDelimiter {
        span: SourceSpan {
            start: Position { row: 0, column: start },
            end: Position { row: 0, column: end },
        },
    }
}

fn create_italic_delimiter(start: usize, end: usize) -> ScannerToken {
    ScannerToken::ItalicDelimiter {
        span: SourceSpan {
            start: Position { row: 0, column: start },
            end: Position { row: 0, column: end },
        },
    }
}

fn create_code_delimiter(start: usize, end: usize) -> ScannerToken {
    ScannerToken::CodeDelimiter {
        span: SourceSpan {
            start: Position { row: 0, column: start },
            end: Position { row: 0, column: end },
        },
    }
}

fn create_text(content: &str, start: usize, end: usize) -> ScannerToken {
    ScannerToken::Text {
        content: content.to_string(),
        span: SourceSpan {
            start: Position { row: 0, column: start },
            end: Position { row: 0, column: end },
        },
    }
}

#[test]
fn test_simple_bold() {
    let tokens = vec![
        create_bold_delimiter(0, 1),
        create_text("hello", 1, 6),
        create_bold_delimiter(6, 7),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();
    assert_eq!(
        result,
        vec![TextTransform::Strong(vec![TextTransform::Identity(
            Text::simple("hello")
        )])]
    );
}

#[test]
fn test_simple_italic() {
    let tokens = vec![
        create_italic_delimiter(0, 1),
        create_text("hello", 1, 6),
        create_italic_delimiter(6, 7),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();
    assert_eq!(
        result,
        vec![TextTransform::Emphasis(vec![TextTransform::Identity(
            Text::simple("hello")
        )])]
    );
}

#[test]
fn test_simple_code() {
    let tokens = vec![
        create_code_delimiter(0, 1),
        create_text("hello", 1, 6),
        create_code_delimiter(6, 7),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();
    assert_eq!(result, vec![TextTransform::Code(Text::simple("hello"))]);
}

#[test]
fn test_nested_bold_italic() {
    let tokens = vec![
        create_bold_delimiter(0, 1),
        create_italic_delimiter(1, 2),
        create_text("hello", 2, 7),
        create_italic_delimiter(7, 8),
        create_bold_delimiter(8, 9),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();
    assert_eq!(
        result,
        vec![TextTransform::Strong(vec![TextTransform::Emphasis(vec![
            TextTransform::Identity(Text::simple("hello"))
        ])])]
    );
}
