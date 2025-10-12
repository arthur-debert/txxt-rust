//! Debug test for centralized regex pattern usage
//!
//! This test verifies that the lexer correctly uses centralized regex patterns
//! from patterns.rs instead of manual string manipulation.

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_reference_marker_patterns() {
        let test_cases = vec![
            // Citation patterns
            ("[@smith2023]", true, "citation"),
            ("[@doe_2024]", true, "citation with underscore"),
            ("[@author-name]", true, "citation with dash"),
            ("[@ ]", false, "invalid citation - space after @"),
            // Section patterns
            ("[#1]", true, "simple section"),
            ("[#1.2.3]", true, "multi-level section"),
            ("[#-1]", true, "negative section index"),
            ("[#1.-1.2]", true, "mixed positive/negative"),
            ("[#]", false, "invalid section - no number"),
            // Footnote patterns
            ("[1]", true, "simple footnote"),
            ("[42]", true, "multi-digit footnote"),
            ("[0]", true, "zero footnote"),
            // URL patterns
            ("[https://example.com]", true, "https url"),
            ("[http://test.org]", true, "http url"),
            ("[www.example.com]", true, "www url"),
            ("[ftp://files.com]", true, "ftp url"),
            // File path patterns
            ("[/path/to/file.txt]", true, "unix path"),
            ("[C:\\Windows\\file.exe]", true, "windows path"),
            ("[document.pdf]", true, "file with extension"),
            ("[config.json]", true, "json file"),
            // Anchor patterns
            ("[my-anchor]", true, "dash anchor"),
            ("[section_2]", true, "underscore anchor"),
            ("[intro.basic]", true, "dotted anchor"),
            ("[Chapter1]", true, "alphanumeric anchor"),
        ];

        for (input, should_be_valid, description) in test_cases {
            println!("Testing {}: '{}'", description, input);

            let tokens = tokenize(input);

            // Find RefMarker tokens
            let ref_markers: Vec<_> = tokens
                .iter()
                .filter(|token| matches!(token, Token::RefMarker { .. }))
                .collect();

            if should_be_valid {
                assert_eq!(
                    ref_markers.len(),
                    1,
                    "Expected 1 RefMarker for valid {}, but got {}: {:?}",
                    description,
                    ref_markers.len(),
                    ref_markers
                );

                if let Token::RefMarker { content, .. } = &ref_markers[0] {
                    // Remove brackets for content comparison
                    let expected_content = &input[1..input.len() - 1];
                    assert_eq!(
                        content, expected_content,
                        "Content mismatch for {}",
                        description
                    );
                    println!("  ✅ Valid: {}", content);
                }
            } else {
                assert_eq!(
                    ref_markers.len(),
                    0,
                    "Expected 0 RefMarkers for invalid {}, but got {}: {:?}",
                    description,
                    ref_markers.len(),
                    ref_markers
                );
                println!("  ✅ Correctly rejected");
            }
        }
    }

    #[test]
    fn debug_regex_consistency() {
        // Test that patterns work the same as before centralization
        let inputs = vec![
            "[@smith2023]",
            "[#1.2.3]",
            "[#-1.1]",
            "[42]",
            "[https://example.com]",
            "[document.pdf]",
            "[my-anchor]",
        ];

        for input in inputs {
            println!("Testing consistency for: {}", input);

            let tokens = tokenize(input);
            let ref_tokens: Vec<_> = tokens
                .iter()
                .filter(|token| matches!(token, Token::RefMarker { .. }))
                .collect();

            assert_eq!(
                ref_tokens.len(),
                1,
                "Should find exactly 1 RefMarker for {}",
                input
            );
            println!("  ✅ Consistent behavior maintained");
        }
    }
}
