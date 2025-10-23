//! Block Grouping Functions
//!
//! Pure functions for grouping tokens into logical blocks.
//! These extracted functions improve testability and maintainability
//! per the progressive-quality-improvements plan (Phase 2, section 3.5).
//!
//! See: docs/proposals/progressive-quality-improvements.txxt

use crate::cst::HighLevelToken;

/// Group consecutive PlainTextLine tokens into a paragraph block
///
/// This function collects consecutive `PlainTextLine` tokens until it encounters
/// a blank line or other element type. This is used to determine paragraph boundaries.
///
/// # Arguments
/// * `tokens` - Slice of high-level tokens to process
/// * `start` - Starting index in the token slice
///
/// # Returns
/// * `Vec<HighLevelToken>` - Collected PlainTextLine tokens
/// * `usize` - Number of tokens consumed
///
/// # Examples
/// ```text
/// Input: [PlainTextLine, PlainTextLine, BlankLine, PlainTextLine]
/// Start: 0
/// Output: ([PlainTextLine, PlainTextLine], 2)
/// ```
pub fn group_contiguous_text_lines(
    tokens: &[HighLevelToken],
    start: usize,
) -> (Vec<HighLevelToken>, usize) {
    let mut grouped_lines = Vec::new();
    let mut position = start;

    while position < tokens.len() {
        let token = &tokens[position];

        match token {
            HighLevelToken::PlainTextLine { .. } => {
                grouped_lines.push(token.clone());
                position += 1;
            }
            HighLevelToken::BlankLine { .. } => {
                // Blank line terminates the group
                break;
            }
            _ => {
                // Any other token terminates the group
                break;
            }
        }
    }

    let consumed = position - start;
    (grouped_lines, consumed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    fn make_plain_text_line(row: usize) -> HighLevelToken {
        HighLevelToken::PlainTextLine {
            indentation_chars: String::new(),
            content: Box::new(HighLevelToken::TextSpan {
                content: format!("line {}", row),
                span: SourceSpan {
                    start: Position { row, column: 0 },
                    end: Position { row, column: 10 },
                },
                tokens: crate::cst::ScannerTokenSequence { tokens: vec![] },
            }),
            span: SourceSpan {
                start: Position { row, column: 0 },
                end: Position { row, column: 10 },
            },
            tokens: crate::cst::ScannerTokenSequence { tokens: vec![] },
        }
    }

    fn make_blank_line(row: usize) -> HighLevelToken {
        HighLevelToken::BlankLine {
            span: SourceSpan {
                start: Position { row, column: 0 },
                end: Position { row, column: 0 },
            },
            tokens: crate::cst::ScannerTokenSequence { tokens: vec![] },
        }
    }

    fn make_indent(row: usize) -> HighLevelToken {
        HighLevelToken::Indent {
            span: SourceSpan {
                start: Position { row, column: 0 },
                end: Position { row, column: 4 },
            },
            tokens: crate::cst::ScannerTokenSequence { tokens: vec![] },
        }
    }

    #[test]
    fn test_group_contiguous_text_lines_single() {
        let tokens = vec![make_plain_text_line(0)];
        let (grouped, consumed) = group_contiguous_text_lines(&tokens, 0);

        assert_eq!(grouped.len(), 1);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_group_contiguous_text_lines_multiple() {
        let tokens = vec![
            make_plain_text_line(0),
            make_plain_text_line(1),
            make_plain_text_line(2),
        ];
        let (grouped, consumed) = group_contiguous_text_lines(&tokens, 0);

        assert_eq!(grouped.len(), 3);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_group_contiguous_text_lines_stops_at_blank() {
        let tokens = vec![
            make_plain_text_line(0),
            make_plain_text_line(1),
            make_blank_line(2),
            make_plain_text_line(3),
        ];
        let (grouped, consumed) = group_contiguous_text_lines(&tokens, 0);

        assert_eq!(grouped.len(), 2);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_group_contiguous_text_lines_stops_at_other_token() {
        let tokens = vec![
            make_plain_text_line(0),
            make_plain_text_line(1),
            make_indent(2),
            make_plain_text_line(3),
        ];
        let (grouped, consumed) = group_contiguous_text_lines(&tokens, 0);

        assert_eq!(grouped.len(), 2);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_group_contiguous_text_lines_from_offset() {
        let tokens = vec![
            make_blank_line(0),
            make_plain_text_line(1),
            make_plain_text_line(2),
        ];
        let (grouped, consumed) = group_contiguous_text_lines(&tokens, 1);

        assert_eq!(grouped.len(), 2);
        assert_eq!(consumed, 2);
    }
}
