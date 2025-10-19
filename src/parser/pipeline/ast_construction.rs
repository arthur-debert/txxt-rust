//! Phase 2b: AST Construction
//!
//! Converts semantic tokens into AST tree nodes using a homegrown grammar engine
//! with carefully planned precedence rules.
//!
//! This phase focuses on the core block elements: Session, Paragraph, List,
//! Annotation, Definition, and Verbatim.

use crate::ast::scanner_tokens::SourceSpan;
use crate::ast::semantic_tokens::{SemanticToken, SemanticTokenList};
use crate::parser::pipeline::BlockParseError;

/// AST Construction parser for converting semantic tokens to AST nodes
///
/// This parser takes semantic tokens and transforms them into structured AST nodes
/// using precedence-based pattern matching.
pub struct AstConstructor {
    /// Current parsing position in the token stream
    position: usize,
    /// Current indentation level for nested parsing
    indentation_level: usize,
}

impl AstConstructor {
    /// Create a new AST constructor instance
    pub fn new() -> Self {
        Self {
            position: 0,
            indentation_level: 0,
        }
    }

    /// Parse semantic tokens into AST nodes
    ///
    /// This is the main entry point for AST construction. It iterates through
    /// the semantic tokens and constructs AST nodes using precedence rules.
    ///
    /// # Arguments
    /// * `semantic_tokens` - The semantic token list to parse
    ///
    /// # Returns
    /// * `Result<Vec<AstNode>, BlockParseError>` - Parsed AST nodes
    pub fn parse(
        &mut self,
        semantic_tokens: &SemanticTokenList,
    ) -> Result<Vec<TempAstNode>, BlockParseError> {
        let mut ast_nodes = Vec::new();

        // Reset parser state
        self.position = 0;
        self.indentation_level = 0;

        // Main parsing loop
        while self.position < semantic_tokens.tokens.len() {
            let token = &semantic_tokens.tokens[self.position];

            // Skip structural tokens that don't contribute to content
            match token {
                SemanticToken::Indent { .. } => {
                    self.indentation_level += 1;
                    self.position += 1;
                    continue;
                }
                SemanticToken::Dedent { .. } => {
                    if self.indentation_level > 0 {
                        self.indentation_level -= 1;
                    }
                    self.position += 1;
                    continue;
                }
                SemanticToken::BlankLine { .. } => {
                    self.position += 1;
                    continue;
                }
                _ => {
                    // Process content tokens
                    if let Some(node) = self.dispatch_parsing(semantic_tokens)? {
                        ast_nodes.push(node);
                    }
                }
            }
        }

        Ok(ast_nodes)
    }

    /// Dispatch parsing based on current token(s) and precedence rules
    ///
    /// This function looks at the current token(s) and decides which specific
    /// parsing function to call based on precedence rules.
    ///
    /// # Arguments
    /// * `semantic_tokens` - The semantic token list
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Parsed node or None
    fn dispatch_parsing(
        &mut self,
        semantic_tokens: &SemanticTokenList,
    ) -> Result<Option<TempAstNode>, BlockParseError> {
        if self.position >= semantic_tokens.tokens.len() {
            return Ok(None);
        }

        let current_token = &semantic_tokens.tokens[self.position];

        // Apply precedence rules (first match wins):
        // 1. VerbatimBlock pattern (highest precedence)
        if let Some(node) = self.try_parse_verbatim_block(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some(node));
        }

        // 2. Annotation pattern
        if let Some(node) = self.try_parse_annotation(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some(node));
        }

        // 3. Definition pattern
        if let Some(node) = self.try_parse_definition(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some(node));
        }

        // 4. Session pattern
        if let Some(node) = self.try_parse_session(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some(node));
        }

        // 5. List pattern
        if let Some(node) = self.try_parse_list(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some(node));
        }

        // 6. Paragraph pattern (catch-all, lowest precedence)
        if let Some(node) = self.try_parse_paragraph(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some(node));
        }

        // No pattern matched
        Ok(None)
    }

    /// Check if we're at the end of the token stream
    #[allow(dead_code)] // Will be used in next steps
    fn is_at_end(&self, semantic_tokens: &SemanticTokenList) -> bool {
        self.position >= semantic_tokens.tokens.len()
    }

    /// Get current token without advancing position
    #[allow(dead_code)] // Will be used in next steps
    fn peek<'a>(&self, semantic_tokens: &'a SemanticTokenList) -> Option<&'a SemanticToken> {
        semantic_tokens.tokens.get(self.position)
    }

    /// Get current token and advance position
    #[allow(dead_code)] // Will be used in next steps
    fn consume(&mut self) -> Option<SemanticToken> {
        // This will be implemented when we have actual tokens to consume
        None
    }

    /// Try to parse an annotation semantic token
    ///
    /// Annotations are already constructed semantic tokens, so we just need to
    /// extract the information and create a temporary AST node.
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Annotation node if matched
    fn try_parse_annotation(
        &self,
        token: &SemanticToken,
    ) -> Result<Option<TempAstNode>, BlockParseError> {
        match token {
            SemanticToken::Annotation {
                label,
                content,
                span,
                ..
            } => {
                // Extract label text
                let label_text = match label.as_ref() {
                    SemanticToken::Label { text, .. } => text.clone(),
                    _ => "unknown".to_string(),
                };

                // Extract content text if present
                let content_text = match content {
                    Some(content_token) => match content_token.as_ref() {
                        SemanticToken::TextSpan { content, .. } => Some(content.clone()),
                        SemanticToken::PlainTextLine { content, .. } => match content.as_ref() {
                            SemanticToken::TextSpan { content, .. } => Some(content.clone()),
                            _ => None,
                        },
                        _ => None,
                    },
                    None => None,
                };

                Ok(Some(TempAstNode::Annotation {
                    label: label_text,
                    content: content_text,
                    span: span.clone(),
                }))
            }
            _ => Ok(None),
        }
    }

    /// Try to parse a verbatim block semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Verbatim block node if matched
    fn try_parse_verbatim_block(
        &self,
        token: &SemanticToken,
    ) -> Result<Option<TempAstNode>, BlockParseError> {
        match token {
            SemanticToken::VerbatimBlock {
                title, label, span, ..
            } => {
                // Extract title text
                let title_text = match title.as_ref() {
                    SemanticToken::TextSpan { content, .. } => content.clone(),
                    _ => "unknown".to_string(),
                };

                // Extract label text
                let label_text = match label.as_ref() {
                    SemanticToken::Label { text, .. } => text.clone(),
                    _ => "unknown".to_string(),
                };

                Ok(Some(TempAstNode::VerbatimBlock {
                    title: title_text,
                    label: label_text,
                    span: span.clone(),
                }))
            }
            _ => Ok(None),
        }
    }

    /// Try to parse a definition semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Definition node if matched
    fn try_parse_definition(
        &self,
        token: &SemanticToken,
    ) -> Result<Option<TempAstNode>, BlockParseError> {
        match token {
            SemanticToken::Definition {
                term,
                parameters,
                span,
                ..
            } => {
                // Extract term text
                let term_text = match term.as_ref() {
                    SemanticToken::TextSpan { content, .. } => content.clone(),
                    _ => "unknown".to_string(),
                };

                // Extract parameters text if present
                let params_text = match parameters {
                    Some(params_token) => {
                        match params_token.as_ref() {
                            SemanticToken::Parameters { params, .. } => {
                                // Convert parameters to string representation
                                let param_strings: Vec<String> =
                                    params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                                Some(param_strings.join(","))
                            }
                            _ => None,
                        }
                    }
                    None => None,
                };

                Ok(Some(TempAstNode::Definition {
                    term: term_text,
                    parameters: params_text,
                    span: span.clone(),
                }))
            }
            _ => Ok(None),
        }
    }

    /// Try to parse a session semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Session node if matched
    fn try_parse_session(
        &self,
        _token: &SemanticToken,
    ) -> Result<Option<TempAstNode>, BlockParseError> {
        // TODO: Implement session parsing
        // Sessions are not simple semantic tokens, they need complex pattern recognition
        Ok(None)
    }

    /// Try to parse a list semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - List node if matched
    fn try_parse_list(
        &self,
        _token: &SemanticToken,
    ) -> Result<Option<TempAstNode>, BlockParseError> {
        // TODO: Implement list parsing
        // Lists are not simple semantic tokens, they need complex pattern recognition
        Ok(None)
    }

    /// Try to parse a paragraph semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Paragraph node if matched
    fn try_parse_paragraph(
        &self,
        token: &SemanticToken,
    ) -> Result<Option<TempAstNode>, BlockParseError> {
        match token {
            SemanticToken::PlainTextLine { content, span, .. } => {
                // Extract content text
                let content_text = match content.as_ref() {
                    SemanticToken::TextSpan { content, .. } => content.clone(),
                    _ => "unknown".to_string(),
                };

                Ok(Some(TempAstNode::Paragraph {
                    content: content_text,
                    span: span.clone(),
                }))
            }
            _ => Ok(None),
        }
    }
}

impl Default for AstConstructor {
    fn default() -> Self {
        Self::new()
    }
}

/// Temporary AST node for testing and development
///
/// This is a placeholder structure that will be replaced with real AST nodes
/// once the identification logic is proven to work correctly.
#[derive(Debug, Clone, PartialEq)]
pub enum TempAstNode {
    /// Placeholder for annotation nodes
    Annotation {
        label: String,
        content: Option<String>,
        span: SourceSpan,
    },
    /// Placeholder for definition nodes
    Definition {
        term: String,
        parameters: Option<String>,
        span: SourceSpan,
    },
    /// Placeholder for verbatim block nodes
    VerbatimBlock {
        title: String,
        label: String,
        span: SourceSpan,
    },
    /// Placeholder for session nodes
    Session {
        title: String,
        child_count: usize,
        span: SourceSpan,
    },
    /// Placeholder for list nodes
    List { item_count: usize, span: SourceSpan },
    /// Placeholder for paragraph nodes
    Paragraph { content: String, span: SourceSpan },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::scanner_tokens::{Position, SourceSpan};
    use crate::ast::semantic_tokens::{SemanticTokenBuilder, SemanticTokenList};

    /// Test that the parser machinery initializes correctly
    #[test]
    fn test_parser_initialization() {
        let parser = AstConstructor::new();
        assert_eq!(parser.position, 0);
        assert_eq!(parser.indentation_level, 0);
    }

    /// Test that the parser handles empty semantic token list
    #[test]
    fn test_parse_empty_tokens() {
        let mut parser = AstConstructor::new();
        let empty_tokens = SemanticTokenList::with_tokens(vec![]);

        let result = parser.parse(&empty_tokens);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Test that the parser skips structural tokens correctly
    #[test]
    fn test_parse_structural_tokens_only() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 4 },
        };

        let tokens = vec![
            SemanticTokenBuilder::indent(span.clone()),
            SemanticTokenBuilder::blank_line(span.clone()),
            SemanticTokenBuilder::dedent(span.clone()),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Test that the parser handles indentation level tracking
    #[test]
    fn test_indentation_level_tracking() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 4 },
        };

        let tokens = vec![
            SemanticTokenBuilder::indent(span.clone()),
            SemanticTokenBuilder::indent(span.clone()),
            SemanticTokenBuilder::dedent(span.clone()),
            SemanticTokenBuilder::dedent(span.clone()),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        // Parser should end with indentation level back to 0
        assert_eq!(parser.indentation_level, 0);
    }

    /// Test that the parser can parse annotation semantic tokens
    #[test]
    fn test_parse_annotation() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 20 },
        };

        let label_span = SourceSpan {
            start: Position { row: 1, column: 3 },
            end: Position { row: 1, column: 7 },
        };

        let content_span = SourceSpan {
            start: Position { row: 1, column: 11 },
            end: Position { row: 1, column: 20 },
        };

        // Create an annotation semantic token
        let annotation_token = SemanticTokenBuilder::annotation(
            SemanticTokenBuilder::label("note".to_string(), label_span),
            None, // No parameters
            Some(SemanticTokenBuilder::text_span(
                "This is a note".to_string(),
                content_span,
            )),
            span,
        );

        let tokens = vec![annotation_token];
        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            TempAstNode::Annotation {
                label,
                content,
                span: node_span,
            } => {
                assert_eq!(label, "note");
                assert_eq!(content, &Some("This is a note".to_string()));
                assert_eq!(node_span.start.row, 1);
                assert_eq!(node_span.start.column, 0);
            }
            _ => panic!("Expected Annotation node, got {:?}", ast_nodes[0]),
        }
    }
}
