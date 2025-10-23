//! Tests for session title numbering extraction (issue #122)
//!
//! Sessions should extract numbering from sequence markers like lists do,
//! but plain markers (-) are NOT valid for sessions.

use txxt::ast::elements::list::{NumberingForm, NumberingStyle};
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::ast::elements::session::SessionNumbering;
use txxt::transform::run_all;

#[test]
fn test_session_simple_numerical() {
    let input = r#"1. Introduction

    This is the content.
"#;
    let doc = run_all(input, Some("test.txxt".to_string())).expect("should parse");

    assert_eq!(doc.content.content.len(), 1);
    if let SessionContainerElement::Session(session) = &doc.content.content[0] {
        // Currently fails: numbering is None
        assert_eq!(
            session.title.numbering,
            Some(SessionNumbering {
                marker: "1.".to_string(),
                style: NumberingStyle::Numerical,
                form: NumberingForm::Short,
            })
        );
    } else {
        panic!("Expected Session, got {:?}", doc.content.content[0]);
    }
}

#[test]
fn test_session_hierarchical_numerical() {
    let input = r#"1.2.3. Deep Section

    Content here.
"#;
    let doc = run_all(input, Some("test.txxt".to_string())).expect("should parse");

    assert_eq!(doc.content.content.len(), 1);
    if let SessionContainerElement::Session(session) = &doc.content.content[0] {
        // Currently fails: numbering is None
        assert_eq!(
            session.title.numbering,
            Some(SessionNumbering {
                marker: "1.2.3.".to_string(),
                style: NumberingStyle::Numerical,
                form: NumberingForm::Full,
            })
        );
    } else {
        panic!("Expected Session");
    }
}

#[test]
fn test_session_no_numbering() {
    let input = r#"Introduction

    This is the content.
"#;
    let doc = run_all(input, Some("test.txxt".to_string())).expect("should parse");

    assert_eq!(doc.content.content.len(), 1);
    if let SessionContainerElement::Session(session) = &doc.content.content[0] {
        // This should already work - no numbering
        assert_eq!(session.title.numbering, None);
    } else {
        panic!("Expected Session");
    }
}

#[test]
fn test_session_plain_marker_invalid() {
    // Plain markers like "-" are valid for lists but NOT for sessions
    // This should parse as a list, not a session
    let input = r#"- Not a session
- Another item
"#;
    let doc = run_all(input, Some("test.txxt".to_string())).expect("should parse");

    assert_eq!(doc.content.content.len(), 1);
    // Should be a List, not a Session
    assert!(matches!(
        doc.content.content[0],
        SessionContainerElement::List(_)
    ));
}
