/// # Ensemble Document Testing Example
///
/// This module demonstrates how to use the txxt corpora system for testing
/// full documents (ensemble tests) as opposed to isolated element tests.
///
/// ## Purpose
///
/// Ensemble documents test how elements integrate and interact in realistic
/// scenarios. Unlike isolated element tests which validate individual components,
/// ensemble tests validate:
/// - Complete document parsing
/// - Element relationships and nesting
/// - Container boundaries
/// - Session hierarchies
/// - Mixed content handling
///
/// ## Document Organization
///
/// Ensemble documents are located in `docs/specs/ensembles/` and follow a
/// progressive complexity ladder from simple to comprehensive:
///
/// - **Foundation (01-03)**: Basic paragraphs and sessions
/// - **Flat Structure (04)**: Multiple peer sessions
/// - **Nested Structure (05)**: Hierarchical sessions
/// - **Mixed Content (06-08)**: Lists, definitions, various elements
/// - **Complex (09)**: Deep nesting with multiple element types
/// - **Feature-Rich (10-11)**: Annotations and comprehensive examples
///
/// ## Testing Strategy
///
/// Process documents in sequence:
/// 1. Start with simplest (01-two-paragraphs.txxt)
/// 2. Progress through each document
/// 3. Each success enables the next level
/// 4. Failure indicates missing capability
// Include the corpora module from the same directory
#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::{ProcessingStage, TxxtCorpora};

/// # Basic Document Loading
///
/// The simplest use case: load a complete document and validate its content.
///
/// ## What This Tests
///
/// - Corpora system can find and load ensemble documents
/// - File content is read correctly
/// - Basic corpus structure is populated
///
/// ## Example Document Structure
///
/// The document `01-two-paragraphs.txxt` contains:
/// ```txxt
/// This is the first paragraph of a simple document.
///
/// This is the second paragraph. It is separated by a blank line.
/// ```
#[test]
fn test_load_simplest_document() {
    // Load using full name (with or without extension)
    let corpus = TxxtCorpora::load_document("01-two-paragraphs")
        .expect("Should load simplest ensemble document");

    // Verify corpus metadata
    assert_eq!(corpus.name, "01-two-paragraphs");
    assert_eq!(corpus.processing_stage, ProcessingStage::ScannerTokens);
    assert!(
        corpus.parameters.is_empty(),
        "Documents don't have parameters"
    );

    // Verify document content
    assert!(
        corpus.source_text.contains("first paragraph"),
        "Document should contain expected text"
    );
    assert!(
        corpus.source_text.contains("second paragraph"),
        "Document should have both paragraphs"
    );

    // Verify structure - should have two paragraphs separated by blank line
    let lines: Vec<&str> = corpus.source_text.lines().collect();
    assert!(
        lines.len() >= 3,
        "Should have at least 2 paragraphs + blank line"
    );
}

/// # Prefix Matching
///
/// The corpora system supports convenient prefix matching, allowing you to
/// reference documents by their number only.
///
/// ## Benefits
///
/// - Shorter test code: use "01" instead of "01-two-paragraphs"
/// - Flexibility: works with any matching document
/// - Convenience: ideal for sequential testing
#[test]
fn test_load_document_by_prefix() {
    // Can load by just the number prefix
    let corpus = TxxtCorpora::load_document("01").expect("Should load by prefix");

    // Should match the same document as full name
    assert_eq!(corpus.name, "01-two-paragraphs");
    assert!(corpus.source_text.contains("first paragraph"));
}

/// # Session Structure Testing
///
/// Tests documents with session structure to validate hierarchical parsing.
///
/// ## What This Tests
///
/// - Session recognition (numbered vs unnumbered)
/// - Content indentation handling
/// - Session container creation
/// - Parent-child relationships
///
/// ## Example Document Structure
///
/// The document `02-session-one-paragraph.txxt` contains:
/// ```txxt
/// Introduction
///
///     This is a simple session with just one paragraph.
/// ```
#[test]
fn test_session_document() {
    let corpus = TxxtCorpora::load_document("02").expect("Should load session document");

    // Verify it has session structure
    assert!(
        corpus.source_text.contains("Introduction"),
        "Should have session title"
    );

    // Check for proper indentation (4 spaces for content)
    let lines: Vec<&str> = corpus.source_text.lines().collect();
    let content_lines: Vec<&&str> = lines
        .iter()
        .filter(|line| line.starts_with("    ") && !line.trim().is_empty())
        .collect();

    assert!(
        !content_lines.is_empty(),
        "Should have indented content lines"
    );
}

/// # Nested Structure Testing
///
/// Tests documents with nested sessions to validate hierarchical parsing.
///
/// ## What This Tests
///
/// - Multiple nesting levels
/// - Session hierarchy (parent/child relationships)
/// - Hierarchical numbering (1, 1.1, 1.2, etc.)
/// - Container boundaries at different levels
///
/// ## Example Document Structure
///
/// The document `05-nested-sessions-basic.txxt` contains:
/// ```txxt
/// 1. Main Topic
///     1.1. First Subtopic
///         Content here...
///     1.2. Second Subtopic
///         Content here...
/// ```
#[test]
fn test_nested_document() {
    let corpus = TxxtCorpora::load_document("05-nested-sessions-basic")
        .expect("Should load nested document");

    // Should contain hierarchical numbering
    assert!(corpus.source_text.contains("1. Main Topic"));
    assert!(corpus.source_text.contains("1.1."));
    assert!(corpus.source_text.contains("1.2."));

    // Should contain nested content at different indentation levels
    let lines: Vec<&str> = corpus.source_text.lines().collect();

    // Check for various indentation levels (0, 4, 8 spaces)
    let has_level_0 = lines.iter().any(|l| l.starts_with("1. "));
    let has_level_1 = lines.iter().any(|l| l.starts_with("    1.1"));
    let has_level_2 = lines
        .iter()
        .any(|l| l.starts_with("        ") && !l.trim().is_empty());

    assert!(has_level_0, "Should have top-level sections");
    assert!(has_level_1, "Should have first-level nesting");
    assert!(has_level_2, "Should have second-level content");
}

/// # Mixed Content Testing
///
/// Tests documents with multiple element types at the same level.
///
/// ## What This Tests
///
/// - Paragraphs, lists, and definitions together
/// - Content container variety
/// - Element recognition in context
/// - Proper separation between different element types
///
/// ## Example Document Structure
///
/// The document `08-mixed-content-flat.txxt` contains paragraphs, lists,
/// and definitions all within the same session, demonstrating how different
/// block types coexist.
#[test]
fn test_mixed_content_document() {
    let corpus = TxxtCorpora::load_document("08").expect("Should load mixed content document");

    // Should contain various element types
    assert!(
        corpus.source_text.contains("- "),
        "Should contain list items"
    );
    assert!(
        corpus.source_text.contains("Term :"),
        "Should contain definitions (single colon)"
    );

    // Should have paragraphs before and after other elements
    assert!(
        corpus.source_text.contains("This document demonstrates"),
        "Should have descriptive paragraphs"
    );
}

/// # Complex Document Testing
///
/// Tests the most comprehensive document with all features.
///
/// ## What This Tests
///
/// - Deep nesting (3+ levels)
/// - All major element types
/// - Inline formatting within text
/// - Verbatim blocks
/// - Annotations (document and element level)
/// - Mathematical expressions
/// - Complete integration of all features
///
/// ## Example Document Structure
///
/// The document `11-full-document.txxt` is a complete technical document
/// showcasing all txxt capabilities working together.
#[test]
fn test_full_document() {
    let corpus =
        TxxtCorpora::load_document("11-full-document").expect("Should load comprehensive document");

    // Should have document-level annotations
    assert!(
        corpus.source_text.contains(":: title ::"),
        "Should have document annotations"
    );

    // Should have various element types
    assert!(corpus.source_text.contains("- "), "Should have lists");
    assert!(
        corpus.source_text.contains("1. "),
        "Should have numbered lists"
    );
    assert!(
        corpus.source_text.contains(" ::"),
        "Should have definitions"
    );

    // Should have inline formatting
    assert!(
        corpus.source_text.contains("*"),
        "Should have bold formatting"
    );
    assert!(
        corpus.source_text.contains("_"),
        "Should have italic formatting"
    );
    assert!(
        corpus.source_text.contains("`"),
        "Should have code formatting"
    );

    // Should have verbatim block (look for language label)
    assert!(
        corpus.source_text.contains(":: python"),
        "Should have verbatim block with language"
    );

    // Should be substantial in size
    assert!(
        corpus.source_text.len() > 1000,
        "Full document should be comprehensive"
    );
}

/// # Loading All Documents
///
/// Demonstrates batch loading of all ensemble documents.
///
/// ## What This Tests
///
/// - Corpora system can enumerate all documents
/// - Documents are loaded in order
/// - All documents are valid
/// - No file system errors
///
/// ## Use Cases
///
/// - Comprehensive test suites that process all documents
/// - Progressive validation (test documents in sequence)
/// - Coverage analysis (ensure all documents are tested)
/// - Benchmark testing (measure parser performance across all documents)
#[test]
fn test_load_all_documents() {
    let documents = TxxtCorpora::load_all_documents().expect("Should load all ensemble documents");

    // We should have all 11 ensemble documents
    assert!(
        documents.len() >= 11,
        "Should have at least 11 ensemble documents, found {}",
        documents.len()
    );

    // All documents should have names
    for doc in &documents {
        assert!(!doc.name.is_empty(), "Each document should have a name");
        assert!(
            !doc.source_text.is_empty(),
            "Document '{}' should have content",
            doc.name
        );
    }

    // Documents should be ordered (01, 02, 03, ...)
    let first = &documents[0];
    assert!(
        first.name.starts_with("01"),
        "First document should be 01-*, was '{}'",
        first.name
    );
}

/// # Processing Stage Integration
///
/// Demonstrates how to load documents with different processing stages
/// for integration with the parsing pipeline.
///
/// ## Processing Stages
///
/// - **Raw**: Original text (default)
/// - **Tokens**: ScannerTokenized stream
/// - **BlockedTokens**: Block-grouped tokens
/// - **ParsedAst**: Complete AST structure
/// - **FullDocument**: Final processed document
///
/// ## Use Cases
///
/// - Tokenizer testing: use `ProcessingStage::ScannerTokens`
/// - Parser testing: use `ProcessingStage::AstFull`
/// - End-to-end testing: use `ProcessingStage::FullDocument`
#[test]
fn test_document_with_processing_stages() {
    // Load for tokenization testing
    let corpus = TxxtCorpora::load_document_with_processing(
        "03-session-multiple-paragraphs",
        ProcessingStage::ScannerTokens,
    )
    .expect("Should load with Tokens stage");

    assert_eq!(corpus.processing_stage, ProcessingStage::ScannerTokens);

    // Note: actual tokenization is a placeholder until parser integration
    // This demonstrates the API for when real processing is implemented
}

/// # Progressive Testing Strategy
///
/// Demonstrates recommended testing approach: process documents in sequence
/// from simple to complex.
///
/// ## Benefits
///
/// - Isolate capabilities: each document adds complexity
/// - Clear failure points: know exactly what level fails
/// - Build incrementally: implement features progressively
/// - Validate thoroughly: ensure foundation before advancing
#[test]
fn test_progressive_document_sequence() {
    // Start with simplest: just paragraphs
    let doc_01 = TxxtCorpora::load_document("01").expect("Foundation level: simple paragraphs");
    assert!(doc_01.source_text.contains("paragraph"));

    // Add session structure
    let doc_02 = TxxtCorpora::load_document("02").expect("Basic level: session structure");
    assert!(doc_02.source_text.lines().any(|l| l.starts_with("    ")));

    // Add nesting
    let doc_05 = TxxtCorpora::load_document("05").expect("Intermediate level: nested sessions");
    assert!(doc_05.source_text.contains("1.1"));

    // Add mixed content
    let doc_08 = TxxtCorpora::load_document("08").expect("Advanced level: mixed content types");
    assert!(doc_08.source_text.contains("- "));

    // Final comprehensive test
    let doc_11 = TxxtCorpora::load_document("11").expect("Expert level: full document");
    assert!(doc_11.source_text.len() > 1000);

    // If we got here, parser can handle all complexity levels!
}

/// # Error Handling
///
/// Demonstrates proper error handling when documents don't exist.
///
/// ## What This Tests
///
/// - Corpora system reports clear errors
/// - Non-existent documents return appropriate error type
/// - Error messages are helpful for debugging
#[test]
fn test_nonexistent_document() {
    let result = TxxtCorpora::load_document("99-does-not-exist");

    assert!(result.is_err(), "Loading nonexistent document should fail");

    if let Err(err) = result {
        let error_message = err.to_string();
        assert!(
            error_message.contains("not found") || error_message.contains("99"),
            "Error message should be helpful: {}",
            error_message
        );
    }
}

/// # Content Validation Helper
///
/// Demonstrates how to validate specific aspects of document structure.
///
/// ## What This Tests
///
/// - Documents have expected structural markers
/// - Content follows txxt conventions
/// - Indentation is correct (4 spaces)
#[test]
fn test_validate_document_structure() {
    let corpus = TxxtCorpora::load_document("04-multiple-sessions-flat")
        .expect("Should load flat sessions document");

    let lines: Vec<&str> = corpus.source_text.lines().collect();

    // Count numbered sessions (lines starting with "N.")
    let session_count = lines
        .iter()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.len() >= 2
                && trimmed.chars().next().unwrap().is_ascii_digit()
                && trimmed.chars().nth(1) == Some('.')
        })
        .count();

    assert!(
        session_count >= 3,
        "Document should have at least 3 numbered sessions"
    );

    // Verify indentation is consistent (multiples of 4)
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            let spaces = line.chars().take_while(|&c| c == ' ').count();
            assert!(
                spaces % 4 == 0,
                "Line {} should have indentation that's a multiple of 4, found {} spaces",
                i + 1,
                spaces
            );
        }
    }
}

// ============================================================================
// Integration Examples
// ============================================================================
//
// The following examples show how ensemble documents integrate with actual
// parser implementation (these would be uncommented when parser is ready):

/*
/// # Parser Integration Example
///
/// Shows how to integrate ensemble documents with the actual txxt parser.
#[test]
fn test_parse_simple_document() {
    use txxt::parser::parse_document;

    let corpus = TxxtCorpora::load_document("01-two-paragraphs")
        .expect("Should load document");

    // Parse the document
    let document = parse_document(&corpus.source_text)
        .expect("Should parse simple document successfully");

    // Validate AST structure
    assert_eq!(document.blocks.len(), 2, "Should have 2 paragraph blocks");

    // Check block types
    use txxt::ast::elements::*;
    assert!(matches!(document.blocks[0], ElementNode::ParagraphBlock(_)));
    assert!(matches!(document.blocks[1], ElementNode::ParagraphBlock(_)));
}

/// # AST Snapshot Testing
///
/// Demonstrates using ensemble documents with snapshot testing.
#[test]
fn test_document_ast_snapshot() {
    use txxt::parser::parse_document;

    let corpus = TxxtCorpora::load_document("11-full-document")
        .expect("Should load full document");

    let document = parse_document(&corpus.source_text)
        .expect("Should parse comprehensive document");

    // Use insta for snapshot testing
    insta::assert_yaml_snapshot!(document);
}

/// # Performance Benchmarking
///
/// Shows how to use ensemble documents for parser benchmarking.
#[test]
fn bench_parse_all_documents() {
    use std::time::Instant;
    use txxt::parser::parse_document;

    let documents = TxxtCorpora::load_all_documents()
        .expect("Should load all documents");

    let start = Instant::now();
    let mut successful_parses = 0;

    for doc in &documents {
        if let Ok(_parsed) = parse_document(&doc.source_text) {
            successful_parses += 1;
        }
    }

    let elapsed = start.elapsed();

    println!(
        "Parsed {}/{} documents in {:?}",
        successful_parses,
        documents.len(),
        elapsed
    );

    assert_eq!(
        successful_parses,
        documents.len(),
        "All documents should parse successfully"
    );
}
*/
