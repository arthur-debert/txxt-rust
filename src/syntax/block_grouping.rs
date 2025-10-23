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

/// Find the end position of an indented block
///
/// Searches forward from start position to find where an indented block ends.
/// A block ends when we encounter a dedent token, blank line, or EOF.
///
/// # Arguments
/// * `tokens` - Slice of high-level tokens to search
/// * `start` - Starting index to search from
///
/// # Returns
/// * `usize` - Index where the block ends (exclusive)
///
/// # Examples
/// ```text
/// Input: [Indent, TextLine, TextLine, Dedent, TextLine]
/// Start: 0
/// Output: 3 (position of Dedent)
/// ```
pub fn find_block_end(tokens: &[HighLevelToken], start: usize) -> usize {
    let mut position = start;

    while position < tokens.len() {
        match &tokens[position] {
            HighLevelToken::Dedent { .. } => {
                // Found the end of the block
                return position;
            }
            HighLevelToken::BlankLine { .. } => {
                // Blank line can also terminate blocks in some contexts
                return position;
            }
            _ => {
                position += 1;
            }
        }
    }

    // Reached EOF - block ends here
    position
}

/// Collect all tokens until a dedent token is found
///
/// This function extracts tokens from start position up to (but not including)
/// the next dedent token. Used for collecting content within an indented block.
///
/// # Arguments
/// * `tokens` - Slice of high-level tokens to process
/// * `start` - Starting index in the token slice
///
/// # Returns
/// * `Vec<HighLevelToken>` - Collected tokens (excluding the dedent)
/// * `usize` - Number of tokens consumed (including the dedent)
///
/// # Examples
/// ```text
/// Input: [TextLine, TextLine, Dedent, TextLine]
/// Start: 0
/// Output: ([TextLine, TextLine], 3)  // Consumed includes the dedent
/// ```
pub fn collect_until_dedent(
    tokens: &[HighLevelToken],
    start: usize,
) -> (Vec<HighLevelToken>, usize) {
    let mut collected = Vec::new();
    let mut position = start;

    while position < tokens.len() {
        match &tokens[position] {
            HighLevelToken::Dedent { .. } => {
                // Found dedent - stop collecting but count it as consumed
                position += 1;
                break;
            }
            token => {
                collected.push(token.clone());
                position += 1;
            }
        }
    }

    let consumed = position - start;
    (collected, consumed)
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

    fn make_dedent(row: usize) -> HighLevelToken {
        HighLevelToken::Dedent {
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

    #[test]
    fn test_find_block_end_at_dedent() {
        let tokens = vec![
            make_indent(0),
            make_plain_text_line(1),
            make_plain_text_line(2),
            make_dedent(3),
            make_plain_text_line(4),
        ];
        let end = find_block_end(&tokens, 0);

        assert_eq!(end, 3); // Dedent position
    }

    #[test]
    fn test_find_block_end_at_blank_line() {
        let tokens = vec![
            make_indent(0),
            make_plain_text_line(1),
            make_blank_line(2),
            make_plain_text_line(3),
        ];
        let end = find_block_end(&tokens, 0);

        assert_eq!(end, 2); // BlankLine position
    }

    #[test]
    fn test_find_block_end_at_eof() {
        let tokens = vec![
            make_indent(0),
            make_plain_text_line(1),
            make_plain_text_line(2),
        ];
        let end = find_block_end(&tokens, 0);

        assert_eq!(end, 3); // EOF (tokens.len())
    }

    #[test]
    fn test_collect_until_dedent_with_content() {
        let tokens = vec![
            make_plain_text_line(0),
            make_plain_text_line(1),
            make_dedent(2),
            make_plain_text_line(3),
        ];
        let (collected, consumed) = collect_until_dedent(&tokens, 0);

        assert_eq!(collected.len(), 2);
        assert_eq!(consumed, 3); // Includes the dedent
    }

    #[test]
    fn test_collect_until_dedent_immediate() {
        let tokens = vec![make_dedent(0), make_plain_text_line(1)];
        let (collected, consumed) = collect_until_dedent(&tokens, 0);

        assert_eq!(collected.len(), 0);
        assert_eq!(consumed, 1); // Just the dedent
    }

    #[test]
    fn test_collect_until_dedent_eof() {
        let tokens = vec![make_plain_text_line(0), make_plain_text_line(1)];
        let (collected, consumed) = collect_until_dedent(&tokens, 0);

        assert_eq!(collected.len(), 2);
        assert_eq!(consumed, 2); // All tokens until EOF
    }
}
