//! Comprehensive tests for reference type classification
//!
//! Tests each reference type in isolation with valid and invalid inputs,
//! plus full classifier tests with precedence validation.

use txxt::ast::reference_types::{ReferenceClassifier, SimpleReferenceType};

#[cfg(test)]
mod url_reference_tests {
    use super::*;

    #[test]
    fn test_url_with_protocol_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_urls = vec![
            "https://example.com",
            "http://test.org",
            "ftp://files.com",
            "https://sub.domain.co.uk",
            "http://localhost:8080",
            "https://github.com/user/repo",
        ];

        for url in valid_urls {
            assert_eq!(
                classifier.classify(url),
                SimpleReferenceType::Url,
                "URL with protocol should be classified as Url: '{}'",
                url
            );
        }
    }

    #[test]
    fn test_url_with_www_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_urls = vec!["www.example.com", "www.test.org", "www.sub.domain.co.uk"];

        for url in valid_urls {
            assert_eq!(
                classifier.classify(url),
                SimpleReferenceType::Url,
                "www URL should be classified as Url: '{}'",
                url
            );
        }
    }

    #[test]
    fn test_url_domain_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_domains = vec![
            "example.com",
            "test.org",
            "sub.domain.co.uk",
            "a.co",
            "long-domain-name.org",
        ];

        for domain in valid_domains {
            assert_eq!(
                classifier.classify(domain),
                SimpleReferenceType::Url,
                "Domain should be classified as Url: '{}'",
                domain
            );
        }
    }

    #[test]
    fn test_url_email_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_emails = vec![
            "user@domain.com",
            "test.email@example.org",
            "user+tag@domain.co.uk",
            "user_name@test-domain.com",
            "123@numbers.co",
        ];

        for email in valid_emails {
            assert_eq!(
                classifier.classify(email),
                SimpleReferenceType::Url,
                "Email should be classified as Url: '{}'",
                email
            );
        }
    }

    #[test]
    fn test_url_invalid() {
        let classifier = ReferenceClassifier::new();

        let invalid_urls = vec![
            "not-a-url",
            "missing.extension",
            "@no-domain",
            "www.",
            "http://",
            "://missing-protocol",
            "domain.",
            ".extension-only",
        ];

        for invalid in invalid_urls {
            assert_ne!(
                classifier.classify(invalid),
                SimpleReferenceType::Url,
                "Invalid URL should not be classified as Url: '{}'",
                invalid
            );
        }
    }
}

#[cfg(test)]
mod section_reference_tests {
    use super::*;

    #[test]
    fn test_section_simple_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_sections = vec!["#1", "#0", "#42", "#999"];

        for section in valid_sections {
            assert_eq!(
                classifier.classify(section),
                SimpleReferenceType::Section,
                "Simple section should be classified as Section: '{}'",
                section
            );
        }
    }

    #[test]
    fn test_section_multilevel_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_sections = vec!["#1.2", "#1.2.3", "#0.1", "#42.1.999", "#1.2.3.4.5"];

        for section in valid_sections {
            assert_eq!(
                classifier.classify(section),
                SimpleReferenceType::Section,
                "Multi-level section should be classified as Section: '{}'",
                section
            );
        }
    }

    #[test]
    fn test_section_negative_index_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_sections = vec!["#-1", "#-1.1", "#-1.2.3", "#1.-1", "#1.-1.2"];

        for section in valid_sections {
            assert_eq!(
                classifier.classify(section),
                SimpleReferenceType::Section,
                "Negative index section should be classified as Section: '{}'",
                section
            );
        }
    }

    #[test]
    fn test_section_invalid() {
        let classifier = ReferenceClassifier::new();

        let invalid_sections = vec![
            "#",      // No number
            "1",      // Missing #
            "#a",     // Letter instead of number
            "#1.a",   // Letter in second level
            "#1.",    // Trailing dot
            "#.1",    // Leading dot
            "#1.2.",  // Trailing dot
            "#-",     // Just dash
            "#--1",   // Double dash
            "#1.-.2", // Dash without number
        ];

        for invalid in invalid_sections {
            assert_ne!(
                classifier.classify(invalid),
                SimpleReferenceType::Section,
                "Invalid section should not be classified as Section: '{}'",
                invalid
            );
        }
    }
}

#[cfg(test)]
mod footnote_reference_tests {
    use super::*;

    #[test]
    fn test_footnote_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_footnotes = vec!["1", "0", "42", "999", "123456"];

        for footnote in valid_footnotes {
            assert_eq!(
                classifier.classify(footnote),
                SimpleReferenceType::Footnote,
                "Numeric reference should be classified as Footnote: '{}'",
                footnote
            );
        }
    }

    #[test]
    fn test_footnote_invalid() {
        let classifier = ReferenceClassifier::new();

        let invalid_footnotes = vec![
            "a",   // Letter
            "1a",  // Number with letter
            "a1",  // Letter with number
            "1.2", // Decimal
            "1,2", // Comma
            "1 2", // Space in middle
            "",    // Empty
            "#1",  // With hash (should be section)
        ];

        for invalid in invalid_footnotes {
            assert_ne!(
                classifier.classify(invalid),
                SimpleReferenceType::Footnote,
                "Invalid footnote should not be classified as Footnote: '{}'",
                invalid
            );
        }
    }
}

#[cfg(test)]
mod citation_reference_tests {
    use super::*;

    #[test]
    fn test_citation_author_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_citations = vec![
            "@author",
            "@smith2023",
            "@doe_2024",
            "@author-name",
            "@author1,author2",
            "@smith,jones,doe",
        ];

        for citation in valid_citations {
            assert_eq!(
                classifier.classify(citation),
                SimpleReferenceType::Citation,
                "Author citation should be classified as Citation: '{}'",
                citation
            );
        }
    }

    #[test]
    fn test_citation_with_page_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_citations = vec![
            "@author, p.45",
            "@smith2023, p.123",
            "@author,p.45,46",
            "@smith, p.45-203",
        ];

        for citation in valid_citations {
            assert_eq!(
                classifier.classify(citation),
                SimpleReferenceType::Citation,
                "Citation with page should be classified as Citation: '{}'",
                citation
            );
        }
    }

    #[test]
    fn test_citation_page_only_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_page_refs = vec![
            "p.43", "pp.43", "p.43,44", "p.43-100", "pp.1-999", "p.1,2,3",
        ];

        for page_ref in valid_page_refs {
            assert_eq!(
                classifier.classify(page_ref),
                SimpleReferenceType::Citation,
                "Page reference should be classified as Citation: '{}'",
                page_ref
            );
        }
    }

    #[test]
    fn test_citation_invalid() {
        let classifier = ReferenceClassifier::new();

        let invalid_citations = vec![
            "@",            // Just @
            "author",       // Missing @
            "@",            // Empty after @
            "@ ",           // Space after @
            "@author page", // Invalid page format
            "p.",           // Page without number
            "pp.",          // Pages without number
            "page.43",      // Wrong page format
        ];

        for invalid in invalid_citations {
            assert_ne!(
                classifier.classify(invalid),
                SimpleReferenceType::Citation,
                "Invalid citation should not be classified as Citation: '{}'",
                invalid
            );
        }
    }
}

#[cfg(test)]
mod tk_reference_tests {
    use super::*;

    #[test]
    fn test_tk_naked_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_tk = vec![
            "TK", "tk", // Should work with case insensitive
            "Tk", "tK",
        ];

        for tk in valid_tk {
            assert_eq!(
                classifier.classify(tk),
                SimpleReferenceType::ToComeTK,
                "Naked TK should be classified as ToComeTK: '{}'",
                tk
            );
        }
    }

    #[test]
    fn test_tk_with_id_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_tk_ids = vec![
            "TK-1",
            "TK-343",
            "TK-a3",
            "TK-someword",
            "tk-1",                    // Case insensitive
            "TK-a",                    // Single char
            "TK-12345678901234567890", // 20 chars (max)
        ];

        for tk_id in valid_tk_ids {
            assert_eq!(
                classifier.classify(tk_id),
                SimpleReferenceType::ToComeTK,
                "TK with ID should be classified as ToComeTK: '{}'",
                tk_id
            );
        }
    }

    #[test]
    fn test_tk_invalid() {
        let classifier = ReferenceClassifier::new();

        let invalid_tk = vec![
            "TK-",                      // No ID
            "TK- ",                     // Space after dash
            "TK-UPPER",                 // Uppercase letters in ID
            "TK-1A",                    // Uppercase in ID
            "TK-a_b",                   // Underscore not allowed
            "TK-a-b",                   // Dash in ID
            "TK-123456789012345678901", // 21 chars (over limit)
            "TKsomething",              // Missing dash
            "TO-COME",                  // Wrong format
        ];

        for invalid in invalid_tk {
            assert_ne!(
                classifier.classify(invalid),
                SimpleReferenceType::ToComeTK,
                "Invalid TK should not be classified as ToComeTK: '{}'",
                invalid
            );
        }
    }
}

#[cfg(test)]
mod file_reference_tests {
    use super::*;

    #[test]
    fn test_file_relative_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_files = vec![
            "./filename.txxt",
            "../other-dir/file.txxt",
            "./simple.txt",
            "../../../deep/path.md",
            "./file",
            "../dir/",
        ];

        for file in valid_files {
            assert_eq!(
                classifier.classify(file),
                SimpleReferenceType::File,
                "Relative file path should be classified as File: '{}'",
                file
            );
        }
    }

    #[test]
    fn test_file_absolute_valid() {
        let classifier = ReferenceClassifier::new();

        let valid_files = vec![
            "/absolute/path/file.txxt",
            "/root/file.txt",
            "/simple",
            "/usr/bin/something",
            "/",
        ];

        for file in valid_files {
            assert_eq!(
                classifier.classify(file),
                SimpleReferenceType::File,
                "Absolute file path should be classified as File: '{}'",
                file
            );
        }
    }

    #[test]
    fn test_file_invalid() {
        let classifier = ReferenceClassifier::new();

        let invalid_files = vec![
            "filename.txt", // No path indicator
            "dir/file.txt", // Relative but no ./
            "file",         // No path indicator
            "C:\\windows",  // Windows path not recognized (could be NotSure)
        ];

        for invalid in invalid_files {
            assert_ne!(
                classifier.classify(invalid),
                SimpleReferenceType::File,
                "Invalid file path should not be classified as File: '{}'",
                invalid
            );
        }
    }
}

#[cfg(test)]
mod not_sure_reference_tests {
    use super::*;

    #[test]
    fn test_not_sure_fallback() {
        let classifier = ReferenceClassifier::new();

        let not_sure_refs = vec![
            "local-section",
            "my-anchor",
            "section_2",
            "intro.basic",
            "Chapter1",
            "some-identifier",
            "C:\\windows\\path", // Windows path
            "custom-ref-type",
        ];

        for ref_content in not_sure_refs {
            assert_eq!(
                classifier.classify(ref_content),
                SimpleReferenceType::NotSure,
                "Unmatched reference should be classified as NotSure: '{}'",
                ref_content
            );
        }
    }

    #[test]
    fn test_empty_content() {
        let classifier = ReferenceClassifier::new();

        let empty_content = vec!["", "   ", "\t", "\n"];

        for empty in empty_content {
            assert_eq!(
                classifier.classify(empty),
                SimpleReferenceType::NotSure,
                "Empty content should be classified as NotSure: '{:?}'",
                empty
            );
        }
    }
}
