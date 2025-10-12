//! Annotation element tokenization
//!
//! Implements tokenization for annotation elements as defined in
//! docs/specs/elements/annotation.txxt
//!
//! Annotation pattern: :: label :: content or :: label:params :: content

// Re-export the annotation marker reading function from infrastructure
pub use crate::tokenizer::infrastructure::markers::txxt_marker::read_annotation_marker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tokens::Token;
    use crate::tokenizer::infrastructure::lexer::Lexer;

    #[test]
    fn test_annotation_marker_basic() {
        let mut lexer = Lexer::new("::");
        let token = read_annotation_marker(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::AnnotationMarker { content, .. } => {
                assert_eq!(content, "::");
            }
            _ => panic!("Expected AnnotationMarker token"),
        }
    }

    #[test]
    fn test_annotation_marker_not_triple() {
        let mut lexer = Lexer::new(":::");
        let token = read_annotation_marker(&mut lexer);

        assert!(token.is_none());
    }
}
