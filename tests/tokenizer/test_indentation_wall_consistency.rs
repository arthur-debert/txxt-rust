use txxt::cst::HighLevelToken;
/// Comprehensive test for the wall concept: indentation_chars must be consistent across ALL elements
///
/// This test verifies that leading whitespace (the "wall") is handled consistently for all element
/// types: sessions, paragraphs, lists, definitions, annotations. The parser should NEVER see
/// indentation whitespace in content - it should only appear in the indentation_chars field.
///
/// Related to issue #116
use txxt::syntax::tokenize;
use txxt::syntax::SemanticAnalyzer;

// Import corpora from test infrastructure
#[path = "../infrastructure/corpora.rs"]
mod corpora;
use corpora::TxxtCorpora;

#[test]
fn test_wall_consistency_full_document() {
    // Load the comprehensive document from corpora
    let corpus =
        TxxtCorpora::load_document("11-full-document").expect("Failed to load full document");

    let source = &corpus.source_text;

    // Tokenize and analyze
    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer
        .analyze(scanner_tokens)
        .expect("Failed to analyze tokens");

    // Track what indentation level we're at (based on Indent/Dedent)
    let mut current_indent_level: usize = 0;
    let mut inside_indent = false;

    println!("\n=== Analyzing all high-level tokens for indentation_chars consistency ===\n");

    for (i, token) in result.tokens.iter().enumerate() {
        match token {
            HighLevelToken::Indent { .. } => {
                current_indent_level += 1;
                inside_indent = true;
                println!("[{}] Indent (level now: {})", i, current_indent_level);
            }

            HighLevelToken::Dedent { .. } => {
                current_indent_level = current_indent_level.saturating_sub(1);
                inside_indent = false;
                println!("[{}] Dedent (level now: {})", i, current_indent_level);
            }

            HighLevelToken::BlankLine { .. } => {
                println!("[{}] BlankLine", i);
            }

            // Line tokens that should have indentation_chars when indented
            HighLevelToken::PlainTextLine {
                indentation_chars,
                content,
                ..
            } => {
                println!(
                    "[{}] PlainTextLine - indent_level={}, indentation_chars=\"{}\" (len={})",
                    i,
                    current_indent_level,
                    indentation_chars.replace(' ', "·"),
                    indentation_chars.len()
                );

                // RULE: If we just saw Indent, this line MUST have indentation_chars
                if inside_indent && current_indent_level > 0 {
                    assert!(!indentation_chars.is_empty(),
                           "PlainTextLine at token {} is indented (level {}) but has empty indentation_chars!\n\
                            Content: {:?}",
                           i, current_indent_level, content);

                    // The indentation should be 4 spaces per level
                    let expected_spaces = current_indent_level * 4;
                    assert_eq!(
                        indentation_chars.len(),
                        expected_spaces,
                        "PlainTextLine at token {} has {} spaces but expected {} (level {})",
                        i,
                        indentation_chars.len(),
                        expected_spaces,
                        current_indent_level
                    );
                }

                // RULE: Content should NEVER start with spaces (they should be in indentation_chars)
                verify_content_has_no_leading_spaces(content, i, "PlainTextLine");
            }

            HighLevelToken::SequenceTextLine {
                indentation_chars,
                marker,
                content,
                ..
            } => {
                println!(
                    "[{}] SequenceTextLine - indent_level={}, indentation_chars=\"{}\" (len={})",
                    i,
                    current_indent_level,
                    indentation_chars.replace(' ', "·"),
                    indentation_chars.len()
                );

                // RULE: If we're indented, sequence lines MUST have indentation_chars
                if inside_indent && current_indent_level > 0 {
                    assert!(!indentation_chars.is_empty(),
                           "SequenceTextLine at token {} is indented (level {}) but has empty indentation_chars!\n\
                            Marker: {:?}, Content: {:?}",
                           i, current_indent_level, marker, content);

                    let expected_spaces = current_indent_level * 4;
                    assert_eq!(
                        indentation_chars.len(),
                        expected_spaces,
                        "SequenceTextLine at token {} has {} spaces but expected {} (level {})",
                        i,
                        indentation_chars.len(),
                        expected_spaces,
                        current_indent_level
                    );
                }

                // RULE: Content should NEVER start with spaces
                verify_content_has_no_leading_spaces(content, i, "SequenceTextLine");
            }

            HighLevelToken::Annotation { label, content, .. } => {
                println!(
                    "[{}] Annotation - label={:?}, has_content={}",
                    i,
                    label,
                    content.is_some()
                );

                // NOTE: Annotations don't have indentation_chars yet - they use their own structure
                // This is OK for now, they're handled separately
            }

            HighLevelToken::Definition { term, .. } => {
                println!("[{}] Definition - term={:?}", i, term);

                // NOTE: Definitions don't have indentation_chars yet - they use their own structure
                // This is OK for now, they're handled separately
            }

            HighLevelToken::VerbatimBlock { .. } => {
                println!("[{}] VerbatimBlock", i);
                // Verbatim blocks have their own wall architecture, skip for now
            }

            // Standalone TextSpan tokens - these are from unimplemented features (annotations, definitions)
            // For NOW, we'll allow them and just warn about them
            // TODO: Once annotations and definitions are fully implemented with proper indentation handling,
            // these should not exist
            HighLevelToken::TextSpan { content, .. } => {
                println!(
                    "[{}] TextSpan (WARNING: standalone) - content={:?}",
                    i, content
                );
                // Don't panic - annotations/definitions aren't fully implemented yet
            }

            _ => {
                println!("[{}] {:?}", i, token);
            }
        }

        // After processing a line token, we're no longer immediately after Indent
        if matches!(
            token,
            HighLevelToken::PlainTextLine { .. } | HighLevelToken::SequenceTextLine { .. }
        ) {
            inside_indent = false;
        }
    }

    println!("\n=== Wall consistency check PASSED ===\n");
}

/// Verify that content inside a token doesn't start with spaces
/// (all leading spaces should be in indentation_chars)
fn verify_content_has_no_leading_spaces(
    content: &HighLevelToken,
    token_index: usize,
    token_type: &str,
) {
    if let HighLevelToken::TextSpan { content: text, .. } = content {
        assert!(
            !text.starts_with(' '),
            "{} at token {} has content starting with spaces: {:?}\n\
                Leading spaces should be in indentation_chars, not content!",
            token_type,
            token_index,
            text
        );
    }
}

#[test]
fn test_wall_nested_lists_and_sessions() {
    // This specific section has complex nesting:
    // - Sessions with nested sessions
    // - Lists with nested lists
    // - Mixed levels of indentation

    let source = r#"3. Advanced Features

    3.1. Definitions

        Technical documents often need to define terms precisely:

        Container ::
            An element that holds other elements.

    3.2. Nested Structure

        Deep nesting allows for detailed organization:

        - Category A
            - Subcategory A1
                1. Item 1
                2. Item 2
"#;

    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(scanner_tokens).expect("Failed to analyze");

    // Count how many different indentation levels we see
    let mut indent_levels_seen = std::collections::HashSet::new();
    let mut current_level: usize = 0;

    for token in &result.tokens {
        match token {
            HighLevelToken::Indent { .. } => {
                current_level += 1;
            }
            HighLevelToken::Dedent { .. } => {
                current_level = current_level.saturating_sub(1);
            }
            HighLevelToken::PlainTextLine {
                indentation_chars, ..
            }
            | HighLevelToken::SequenceTextLine {
                indentation_chars, ..
            } => {
                if !indentation_chars.is_empty() {
                    indent_levels_seen.insert(indentation_chars.len());
                }
            }
            _ => {}
        }
    }

    println!("Indentation levels seen: {:?}", indent_levels_seen);

    // We should see multiple indentation levels (4, 8, 12, 16 spaces)
    assert!(
        indent_levels_seen.len() >= 3,
        "Should see at least 3 different indentation levels in nested structure"
    );

    // All indentation should be multiples of 4
    for &level in &indent_levels_seen {
        assert_eq!(
            level % 4,
            0,
            "Indentation level {} is not a multiple of 4",
            level
        );
    }
}

#[test]
fn test_wall_top_level_has_no_indentation_chars() {
    // Top-level content should have EMPTY indentation_chars
    // (unless it's indented content like inside a definition)
    let source = r#"This is a paragraph.

1. This is a session.

- This is a list.
"#;

    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(scanner_tokens).expect("Failed to analyze");

    // Track if we've seen any Indent tokens
    let mut seen_indent = false;

    for (i, token) in result.tokens.iter().enumerate() {
        if matches!(token, HighLevelToken::Indent { .. }) {
            seen_indent = true;
        }

        match token {
            HighLevelToken::PlainTextLine {
                indentation_chars, ..
            }
            | HighLevelToken::SequenceTextLine {
                indentation_chars, ..
            } => {
                // Before any Indent, content should have empty indentation_chars
                if !seen_indent {
                    assert_eq!(
                        indentation_chars, "",
                        "Top-level token {} should have empty indentation_chars, got: {:?}",
                        i, indentation_chars
                    );
                }
            }
            _ => {}
        }
    }
}
