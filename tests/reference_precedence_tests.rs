//! Tests for reference type classification precedence
//!
//! These tests verify that the classifier follows the correct precedence order
//! as specified in the TXXT spec when content could match multiple patterns.

use txxt::ast::reference_types::{ReferenceClassifier, SimpleReferenceType};

#[cfg(test)]
mod precedence_tests {
    use super::*;

    #[test]
    fn test_url_takes_precedence_over_section() {
        let classifier = ReferenceClassifier::new();

        // Content that could be interpreted as both URL and section patterns
        let url_wins = vec![
            // These should be URLs, not sections with fragments
            ("example.com", SimpleReferenceType::Url),
            ("test.org", SimpleReferenceType::Url),
        ];

        for (content, expected) in url_wins {
            assert_eq!(
                classifier.classify(content),
                expected,
                "URL pattern should take precedence: '{}'",
                content
            );
        }
    }

    #[test]
    fn test_section_takes_precedence_over_footnote() {
        let classifier = ReferenceClassifier::new();

        // Content starting with # should be section, not footnote
        let section_wins = vec![
            ("#1", SimpleReferenceType::Section),
            ("#42", SimpleReferenceType::Section),
            ("#0", SimpleReferenceType::Section),
        ];

        for (content, expected) in section_wins {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Section pattern should take precedence over footnote: '{}'",
                content
            );
        }
    }

    #[test]
    fn test_footnote_takes_precedence_over_citation() {
        let classifier = ReferenceClassifier::new();

        // Pure numbers should be footnotes, not treated as potential citation components
        let footnote_wins = vec![
            ("1", SimpleReferenceType::Footnote),
            ("42", SimpleReferenceType::Footnote),
            ("999", SimpleReferenceType::Footnote),
        ];

        for (content, expected) in footnote_wins {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Footnote pattern should take precedence: '{}'",
                content
            );
        }
    }

    #[test]
    fn test_citation_takes_precedence_over_tk() {
        let classifier = ReferenceClassifier::new();

        // @ patterns should be citations even if they might contain letters that TK uses
        let citation_wins = vec![
            ("@author", SimpleReferenceType::Citation),
            ("@tk2023", SimpleReferenceType::Citation), // Contains 'tk' but starts with @
            ("p.43", SimpleReferenceType::Citation),
        ];

        for (content, expected) in citation_wins {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Citation pattern should take precedence: '{}'",
                content
            );
        }
    }

    #[test]
    fn test_tk_takes_precedence_over_file() {
        let classifier = ReferenceClassifier::new();

        // TK patterns should be recognized even if they might look like other things
        let tk_wins = vec![
            ("TK", SimpleReferenceType::ToComeTK),
            ("TK-1", SimpleReferenceType::ToComeTK),
            ("tk", SimpleReferenceType::ToComeTK),
        ];

        for (content, expected) in tk_wins {
            assert_eq!(
                classifier.classify(content),
                expected,
                "TK pattern should take precedence: '{}'",
                content
            );
        }
    }

    #[test]
    fn test_file_takes_precedence_over_not_sure() {
        let classifier = ReferenceClassifier::new();

        // File patterns should be recognized over generic anchor patterns
        let file_wins = vec![
            ("./file", SimpleReferenceType::File),
            ("../dir/file", SimpleReferenceType::File),
            ("/absolute/path", SimpleReferenceType::File),
        ];

        for (content, expected) in file_wins {
            assert_eq!(
                classifier.classify(content),
                expected,
                "File pattern should take precedence over NotSure: '{}'",
                content
            );
        }
    }

    #[test]
    fn test_complete_precedence_chain() {
        let classifier = ReferenceClassifier::new();

        // Test cases that demonstrate the full precedence order
        let precedence_cases = vec![
            // URLs win over everything
            ("example.com", SimpleReferenceType::Url),
            ("https://test.org", SimpleReferenceType::Url),
            ("user@domain.com", SimpleReferenceType::Url),
            // Sections win over footnotes and below
            ("#1", SimpleReferenceType::Section),
            ("#1.2.3", SimpleReferenceType::Section),
            ("#-1", SimpleReferenceType::Section),
            // Footnotes win over citations and below
            ("1", SimpleReferenceType::Footnote),
            ("42", SimpleReferenceType::Footnote),
            ("999", SimpleReferenceType::Footnote),
            // Citations win over TK and below
            ("@author", SimpleReferenceType::Citation),
            ("@smith2023", SimpleReferenceType::Citation),
            ("p.43", SimpleReferenceType::Citation),
            // TK wins over files and below
            ("TK", SimpleReferenceType::ToComeTK),
            ("TK-1", SimpleReferenceType::ToComeTK),
            ("tk-someword", SimpleReferenceType::ToComeTK),
            // Files win over NotSure
            ("./file.txt", SimpleReferenceType::File),
            ("../dir/file", SimpleReferenceType::File),
            ("/absolute/path", SimpleReferenceType::File),
            // NotSure is the fallback
            ("local-anchor", SimpleReferenceType::NotSure),
            ("my-section", SimpleReferenceType::NotSure),
            ("unknown-pattern", SimpleReferenceType::NotSure),
        ];

        for (content, expected) in precedence_cases {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Precedence test failed for: '{}'",
                content
            );
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_ambiguous_patterns() {
        let classifier = ReferenceClassifier::new();

        // Test cases where content might be ambiguous but precedence should resolve it
        let ambiguous_cases = vec![
            // Could be anchor or file, but file starts with specific chars
            ("./tk", SimpleReferenceType::File), // Not TK because of ./
            ("./TK", SimpleReferenceType::File), // Not TK because of ./
            // Numbers that might be confused
            ("0", SimpleReferenceType::Footnote), // Pure number = footnote
            ("#0", SimpleReferenceType::Section), // With # = section
            // @ patterns that might be confused
            ("@1", SimpleReferenceType::Citation), // @ makes it citation, not footnote
            ("@TK", SimpleReferenceType::Citation), // @ makes it citation, not TK
            // Domain-like patterns
            ("tk.com", SimpleReferenceType::Url), // .com makes it URL, not TK
            ("file.txt", SimpleReferenceType::NotSure), // .txt is not a valid URL domain
            // Path-like patterns
            ("p.txt", SimpleReferenceType::NotSure), // Could be page ref but no number
            ("pp.txt", SimpleReferenceType::NotSure), // Could be pages ref but no number
        ];

        for (content, expected) in ambiguous_cases {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Ambiguous pattern resolution failed for: '{}'",
                content
            );
        }
    }

    #[test]
    fn test_whitespace_handling() {
        let classifier = ReferenceClassifier::new();

        // Test that whitespace is properly trimmed and handled
        let whitespace_cases = vec![
            (" TK ", SimpleReferenceType::ToComeTK),
            ("\t@author\t", SimpleReferenceType::Citation),
            (" #1 ", SimpleReferenceType::Section),
            ("  42  ", SimpleReferenceType::Footnote),
            (" ./file ", SimpleReferenceType::File),
            ("   example.com   ", SimpleReferenceType::Url),
        ];

        for (content, expected) in whitespace_cases {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Whitespace handling failed for: '{:?}'",
                content
            );
        }
    }

    #[test]
    fn test_case_sensitivity() {
        let classifier = ReferenceClassifier::new();

        // Test case sensitivity rules
        let case_cases = vec![
            // TK should be case insensitive
            ("TK", SimpleReferenceType::ToComeTK),
            ("tk", SimpleReferenceType::ToComeTK),
            ("Tk", SimpleReferenceType::ToComeTK),
            ("tK", SimpleReferenceType::ToComeTK),
            // TK-id should accept lowercase only in the ID part
            ("TK-abc", SimpleReferenceType::ToComeTK),
            ("tk-123", SimpleReferenceType::ToComeTK),
            // But uppercase letters in ID should fail
            ("TK-ABC", SimpleReferenceType::NotSure),
            ("TK-Ab", SimpleReferenceType::NotSure),
            // URLs should be case preserving but domains are typically lowercase
            ("EXAMPLE.COM", SimpleReferenceType::Url),
            ("Example.Com", SimpleReferenceType::Url),
            // Citations preserve case
            ("@Author", SimpleReferenceType::Citation),
            ("@SMITH2023", SimpleReferenceType::Citation),
        ];

        for (content, expected) in case_cases {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Case sensitivity test failed for: '{}'",
                content
            );
        }
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_real_world_examples() {
        let classifier = ReferenceClassifier::new();

        // Real-world examples from TXXT documents
        let real_examples = vec![
            // From documentation
            ("architecture.txxt", SimpleReferenceType::NotSure), // .txxt is not a URL domain
            ("/01-parser-architecture.txxt#1", SimpleReferenceType::File),
            ("example.com", SimpleReferenceType::Url),
            ("http://example.com", SimpleReferenceType::Url),
            // From citations
            ("@smith2023", SimpleReferenceType::Citation),
            ("@doe2024", SimpleReferenceType::Citation),
            ("p.45", SimpleReferenceType::Citation),
            ("pp.43-100", SimpleReferenceType::Citation),
            // From sections
            ("#1", SimpleReferenceType::Section),
            ("#-1.1", SimpleReferenceType::Section),
            ("#3.2.1", SimpleReferenceType::Section),
            // From footnotes
            ("1", SimpleReferenceType::Footnote),
            ("42", SimpleReferenceType::Footnote),
            ("3", SimpleReferenceType::Footnote),
            // From TK references
            ("TK", SimpleReferenceType::ToComeTK),
            ("TK-1", SimpleReferenceType::ToComeTK),
            ("TK-introduction", SimpleReferenceType::ToComeTK),
            // From file references
            ("./filename.txxt", SimpleReferenceType::File),
            ("../other-dir/file.txxt", SimpleReferenceType::File),
            ("/absolute/path/file.txxt", SimpleReferenceType::File),
            // Anchors and other
            ("local-section", SimpleReferenceType::NotSure),
            ("my-anchor", SimpleReferenceType::NotSure),
            ("security-note", SimpleReferenceType::NotSure),
        ];

        for (content, expected) in real_examples {
            assert_eq!(
                classifier.classify(content),
                expected,
                "Real-world example failed for: '{}'",
                content
            );
        }
    }
}
