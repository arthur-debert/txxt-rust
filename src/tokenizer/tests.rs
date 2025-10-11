#[cfg(test)]
mod tokenizer_tests {
    use crate::tokenizer::{tokenize, TokenType};

    #[test]
    fn test_simple_text() {
        let tokens = tokenize("Hello world");
        // Should be TEXT, NEWLINE, EOF
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TokenType::Text);
        assert_eq!(tokens[0].value, Some("Hello world".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Newline);
        assert_eq!(tokens[2].token_type, TokenType::Eof);
    }

    #[test]
    fn test_sequence_marker() {
        let tokens = tokenize("1. First item");

        let sequence_markers: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::SequenceMarker)
            .collect();
        assert_eq!(sequence_markers.len(), 1);
        assert_eq!(sequence_markers[0].value, Some("1. ".to_string()));
    }

    #[test]
    fn test_dash_list() {
        let tokens = tokenize("- List item");

        let dash_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Dash)
            .collect();
        assert_eq!(dash_tokens.len(), 1);
        assert_eq!(dash_tokens[0].value, Some("- ".to_string()));
    }

    #[test]
    fn test_pragma_annotation() {
        let tokens = tokenize(":: title :: My Document");

        let pragma_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::PragmaMarker)
            .collect();
        assert_eq!(pragma_tokens.len(), 2);

        let identifiers: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Identifier)
            .collect();
        assert_eq!(identifiers.len(), 1);
        assert_eq!(identifiers[0].value, Some("title".to_string()));
    }

    #[test]
    fn test_verbatim_block() {
        let text = "Code:\n    print('hello')\n(python)";
        let tokens = tokenize(text);

        let verbatim_start: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::VerbatimStart)
            .collect();
        assert_eq!(verbatim_start.len(), 1);

        let verbatim_content: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::VerbatimContent)
            .collect();
        assert_eq!(verbatim_content.len(), 1);
    }

    #[test]
    fn test_inline_formatting() {
        let tokens = tokenize("This is *bold* text");

        let strong_markers: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::StrongMarker)
            .collect();
        assert_eq!(strong_markers.len(), 2); // Opening and closing
    }

    #[test]
    fn test_references() {
        let tokens = tokenize("See [reference] and [@citation] and [42]");

        let ref_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| {
                matches!(
                    t.token_type,
                    TokenType::RefMarker | TokenType::Citation | TokenType::FootnoteNumber
                )
            })
            .collect();
        assert_eq!(ref_tokens.len(), 3);
    }

    #[test]
    fn test_indentation() {
        let text = "Level 0\n    Level 1\n        Level 2\n    Back to 1\nLevel 0";
        let tokens = tokenize(text);

        let indent_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Indent)
            .collect();
        assert_eq!(indent_tokens.len(), 2); // Level 1 and Level 2

        let dedent_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Dedent)
            .collect();
        assert_eq!(dedent_tokens.len(), 2); // Back to 1 and Level 0
    }

    #[test]
    fn test_blank_lines() {
        let text = "Line 1\n\nLine 3";
        let tokens = tokenize(text);

        let blank_line_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::BlankLine)
            .collect();
        assert_eq!(blank_line_tokens.len(), 1);
    }

    #[test]
    fn test_definition() {
        let tokens = tokenize("Term ::");

        let def_markers: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::DefinitionMarker)
            .collect();
        assert_eq!(def_markers.len(), 1);
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens.len(), 1); // Just EOF
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn test_line_column_tracking() {
        let text = "Line 1\nLine 2";
        let tokens = tokenize(text);

        // Find the second TEXT token (should be "Line 2")
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Text)
            .collect();
        assert_eq!(text_tokens.len(), 2);
        assert_eq!(text_tokens[1].line, 2);
        assert_eq!(text_tokens[1].column, 1);
    }
}
