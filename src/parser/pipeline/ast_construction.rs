//! Phase 2b: AST Construction
//!
//! Converts semantic tokens into AST tree nodes using a homegrown grammar engine
//! with carefully planned precedence rules.
//!
//! This phase focuses on the core block elements: Session, Paragraph, List,
//! Annotation, Definition, and Verbatim.

use crate::ast::scanner_tokens::{Position, SourceSpan};
use crate::ast::semantic_tokens::{SemanticToken, SemanticTokenList};
use crate::parser::pipeline::BlockParseError;

/// AST Construction parser for converting semantic tokens to AST nodes
///
/// This parser takes semantic tokens and transforms them into structured AST nodes
/// using precedence-based pattern matching.
pub struct AstConstructor<'a> {
    /// The semantic token stream being parsed
    tokens: &'a [SemanticToken],
    /// Current parsing position in the token stream
    position: usize,
    /// Current indentation level for nested parsing
    indentation_level: usize,
}

impl<'a> AstConstructor<'a> {
    /// Create a new AST constructor instance
    pub fn new() -> Self {
        Self {
            tokens: &[],
            position: 0,
            indentation_level: 0,
        }
    }

    /// Create a new AST constructor instance with token stream
    pub fn with_tokens(tokens: &'a [SemanticToken]) -> Self {
        Self {
            tokens,
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
    /// * `Result<Vec<TempAstNode>, BlockParseError>` - Parsed AST nodes
    pub fn parse(
        &mut self,
        semantic_tokens: &'a SemanticTokenList,
    ) -> Result<Vec<TempAstNode>, BlockParseError> {
        // Update the token stream
        self.tokens = &semantic_tokens.tokens;
        let mut ast_nodes = Vec::new();

        // Reset parser state
        self.position = 0;
        self.indentation_level = 0;

        // Main parsing loop
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];

            // Handle structural tokens that don't contribute to content
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
                _ => {
                    // Process all content tokens through the dispatcher
                    if let Some((node, _tokens_consumed)) = self.dispatch_parsing()? {
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
    /// # Returns
    /// * `Result<Option<(TempAstNode, usize)>, BlockParseError>` - Parsed node and tokens consumed
    fn dispatch_parsing(&mut self) -> Result<Option<(TempAstNode, usize)>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let current_token = &self.tokens[self.position];

        // Apply precedence rules (first match wins):
        // 1. VerbatimBlock pattern (highest precedence)
        if let Some(node) = self.try_parse_verbatim_block(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some((node, 1)));
        }

        // 2. Annotation pattern
        if let Some(node) = self.try_parse_annotation(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some((node, 1)));
        }

        // 3. Definition pattern
        if let Some(node) = self.try_parse_definition(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some((node, 1)));
        }

        // 4. Session pattern (check for BlankLine and trigger lookahead)
        if matches!(current_token, SemanticToken::BlankLine { .. }) {
            if let Some((node, tokens_consumed)) = self.try_parse_session()? {
                self.position += tokens_consumed;
                return Ok(Some((node, tokens_consumed)));
            }
            // If not a session, skip the blank line
            self.position += 1;
            return Ok(None); // Blank line consumed but no node created
        }

        // 5. List pattern
        if let Some((node, tokens_consumed)) = self.try_parse_list()? {
            self.position += tokens_consumed;
            return Ok(Some((node, tokens_consumed)));
        }

        // 6. Paragraph pattern (catch-all, lowest precedence)
        if let Some(node) = self.try_parse_paragraph(current_token)? {
            self.position += 1; // Consume the token
            return Ok(Some((node, 1)));
        }

        // No pattern matched
        Ok(None)
    }

    /// Check if we're at the end of the token stream
    #[allow(dead_code)] // Will be used in next steps
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Get current token without advancing position
    #[allow(dead_code)] // Will be used in next steps
    fn peek(&self) -> Option<&SemanticToken> {
        self.tokens.get(self.position)
    }

    /// Get current token and advance position
    #[allow(dead_code)] // Will be used in next steps
    fn consume(&mut self) -> Option<&SemanticToken> {
        if self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            self.position += 1;
            Some(token)
        } else {
            None
        }
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

    /// Try to parse a session pattern
    ///
    /// Sessions follow the pattern:
    /// 1. Blank line (mandatory)
    /// 2. Title line (can be plain text, sequence marker, or definition)
    /// 3. Blank line (mandatory)
    /// 4. Indent token
    /// 5. Content block (must have at least one indented child)
    ///
    /// # Returns
    /// * `Result<Option<(TempAstNode, usize)>, BlockParseError>` - Session node and tokens consumed if matched
    fn try_parse_session(&self) -> Result<Option<(TempAstNode, usize)>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        // Look ahead to see if we have a session pattern
        let mut lookahead_pos = self.position;

        // Step 1: Must start with blank line
        if lookahead_pos >= self.tokens.len() {
            return Ok(None);
        }
        let first_token = &self.tokens[lookahead_pos];
        match first_token {
            SemanticToken::BlankLine { .. } => {
                lookahead_pos += 1;
            }
            _ => return Ok(None),
        }

        // Step 2: Must have title line (any content except blank line)
        if lookahead_pos >= self.tokens.len() {
            return Ok(None);
        }
        let title_token = &self.tokens[lookahead_pos];
        let title_text = match title_token {
            SemanticToken::PlainTextLine { content, .. } => match content.as_ref() {
                SemanticToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            SemanticToken::SequenceTextLine { content, .. } => match content.as_ref() {
                SemanticToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            SemanticToken::Definition { term, .. } => match term.as_ref() {
                SemanticToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            _ => return Ok(None), // Not a valid title
        };
        lookahead_pos += 1;

        // Step 3: Must have blank line after title
        if lookahead_pos >= self.tokens.len() {
            return Ok(None);
        }
        let second_blank_token = &self.tokens[lookahead_pos];
        match second_blank_token {
            SemanticToken::BlankLine { .. } => {
                lookahead_pos += 1;
            }
            _ => return Ok(None),
        }

        // Step 4: Must have indent token
        if lookahead_pos >= self.tokens.len() {
            return Ok(None);
        }
        let indent_token = &self.tokens[lookahead_pos];
        match indent_token {
            SemanticToken::Indent { .. } => {
                lookahead_pos += 1;
            }
            _ => return Ok(None),
        }

        // Step 5: Must have at least one indented child
        let mut child_count = 0;
        let mut current_pos = lookahead_pos;
        while current_pos < self.tokens.len() {
            let token = &self.tokens[current_pos];
            match token {
                SemanticToken::Dedent { .. } => break,
                SemanticToken::BlankLine { .. } => {
                    current_pos += 1;
                    continue;
                }
                _ => {
                    child_count += 1;
                    current_pos += 1;
                }
            }
        }

        if child_count == 0 {
            return Ok(None); // No children, not a session
        }

        // We have a valid session pattern!
        let span = match first_token {
            SemanticToken::BlankLine { span } => span.clone(),
            _ => return Ok(None),
        };

        // Calculate how many tokens we consumed
        let tokens_consumed = current_pos - self.position;

        Ok(Some((
            TempAstNode::Session {
                title: title_text,
                child_count,
                span,
            },
            tokens_consumed,
        )))
    }

    /// Try to parse a list pattern
    ///
    /// Lists follow the pattern:
    /// 1. Sequence of at least 2 items with sequence markers
    /// 2. Items cannot have blank lines between them
    /// 3. Items can be nested (but we'll handle simple lists first)
    ///
    /// # Returns
    /// * `Result<Option<(TempAstNode, usize)>, BlockParseError>` - List node and tokens consumed if matched
    fn try_parse_list(&self) -> Result<Option<(TempAstNode, usize)>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        // Look ahead to see if we have a list pattern
        let mut lookahead_pos = self.position;
        let mut item_count = 0;
        let mut has_blank_lines = false;
        let start_pos = lookahead_pos;

        // Count consecutive sequence text lines
        while lookahead_pos < self.tokens.len() {
            let token = &self.tokens[lookahead_pos];
            match token {
                SemanticToken::SequenceTextLine { .. } => {
                    item_count += 1;
                    lookahead_pos += 1;
                }
                SemanticToken::BlankLine { .. } => {
                    // Check if this blank line is between list items
                    if item_count > 0 {
                        // Look ahead to see if there's another sequence marker
                        let mut next_pos = lookahead_pos + 1;
                        while next_pos < self.tokens.len() {
                            let next_token = &self.tokens[next_pos];
                            match next_token {
                                SemanticToken::BlankLine { .. } => {
                                    next_pos += 1;
                                    continue;
                                }
                                SemanticToken::SequenceTextLine { .. } => {
                                    has_blank_lines = true;
                                    break;
                                }
                                _ => break,
                            }
                        }
                    }
                    lookahead_pos += 1;
                }
                _ => break,
            }
        }

        // Lists must have at least 2 items
        if item_count < 2 {
            return Ok(None);
        }

        // Lists cannot have blank lines between items
        if has_blank_lines {
            return Ok(None);
        }

        // We have a valid list pattern!
        let span = match self.tokens.get(start_pos) {
            Some(SemanticToken::SequenceTextLine { span, .. }) => span.clone(),
            _ => {
                // Fallback span
                SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 0 },
                }
            }
        };

        // Calculate how many tokens we consumed
        let tokens_consumed = lookahead_pos - self.position;

        Ok(Some((
            TempAstNode::List { item_count, span },
            tokens_consumed,
        )))
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

impl<'a> Default for AstConstructor<'a> {
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
    use crate::ast::semantic_tokens::{
        SemanticNumberingForm, SemanticNumberingStyle, SemanticTokenBuilder, SemanticTokenList,
    };

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

    /// Test that the parser can parse definition semantic tokens
    #[test]
    fn test_parse_definition() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 15 },
        };

        let term_span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 12 },
        };

        // Create a definition semantic token
        let definition_token = SemanticTokenBuilder::definition(
            SemanticTokenBuilder::text_span("Term".to_string(), term_span),
            None, // No parameters
            span,
        );

        let tokens = vec![definition_token];
        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            TempAstNode::Definition {
                term,
                parameters,
                span: node_span,
            } => {
                assert_eq!(term, "Term");
                assert_eq!(parameters, &None);
                assert_eq!(node_span.start.row, 1);
                assert_eq!(node_span.start.column, 0);
            }
            _ => panic!("Expected Definition node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse paragraph semantic tokens
    #[test]
    fn test_parse_paragraph() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 12 },
        };

        let content_span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 12 },
        };

        // Create a paragraph semantic token
        let paragraph_token = SemanticTokenBuilder::plain_text_line(
            SemanticTokenBuilder::text_span("Hello world".to_string(), content_span),
            span,
        );

        let tokens = vec![paragraph_token];
        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            TempAstNode::Paragraph {
                content,
                span: node_span,
            } => {
                assert_eq!(content, "Hello world");
                assert_eq!(node_span.start.row, 1);
                assert_eq!(node_span.start.column, 0);
            }
            _ => panic!("Expected Paragraph node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse session patterns
    #[test]
    fn test_parse_session() {
        let mut parser = AstConstructor::new();

        let span1 = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 0 },
        };

        let span2 = SourceSpan {
            start: Position { row: 2, column: 0 },
            end: Position { row: 2, column: 10 },
        };

        let span3 = SourceSpan {
            start: Position { row: 3, column: 0 },
            end: Position { row: 3, column: 0 },
        };

        let span4 = SourceSpan {
            start: Position { row: 4, column: 0 },
            end: Position { row: 4, column: 4 },
        };

        let span5 = SourceSpan {
            start: Position { row: 5, column: 4 },
            end: Position { row: 5, column: 20 },
        };

        // Create a session pattern: blank line + title + blank line + indent + content
        let tokens = vec![
            SemanticTokenBuilder::blank_line(span1),
            SemanticTokenBuilder::plain_text_line(
                SemanticTokenBuilder::text_span("Session Title".to_string(), span2.clone()),
                span2,
            ),
            SemanticTokenBuilder::blank_line(span3),
            SemanticTokenBuilder::indent(span4),
            SemanticTokenBuilder::plain_text_line(
                SemanticTokenBuilder::text_span("Session content".to_string(), span5.clone()),
                span5,
            ),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            TempAstNode::Session {
                title,
                child_count,
                span: node_span,
            } => {
                assert_eq!(title, "Session Title");
                assert_eq!(*child_count, 1);
                assert_eq!(node_span.start.row, 1);
                assert_eq!(node_span.start.column, 0);
            }
            _ => panic!("Expected Session node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse list patterns
    #[test]
    fn test_parse_list() {
        let mut parser = AstConstructor::new();

        let span1 = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 10 },
        };

        let span2 = SourceSpan {
            start: Position { row: 2, column: 0 },
            end: Position { row: 2, column: 12 },
        };

        // Create a list pattern: two sequence text lines
        let tokens = vec![
            SemanticTokenBuilder::sequence_text_line(
                SemanticTokenBuilder::sequence_marker(
                    SemanticNumberingStyle::Plain,
                    SemanticNumberingForm::Regular,
                    "-".to_string(),
                    span1.clone(),
                ),
                SemanticTokenBuilder::text_span("First item".to_string(), span1.clone()),
                span1,
            ),
            SemanticTokenBuilder::sequence_text_line(
                SemanticTokenBuilder::sequence_marker(
                    SemanticNumberingStyle::Plain,
                    SemanticNumberingForm::Regular,
                    "-".to_string(),
                    span2.clone(),
                ),
                SemanticTokenBuilder::text_span("Second item".to_string(), span2.clone()),
                span2,
            ),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            TempAstNode::List {
                item_count,
                span: node_span,
            } => {
                assert_eq!(*item_count, 2);
                assert_eq!(node_span.start.row, 1);
                assert_eq!(node_span.start.column, 0);
            }
            _ => panic!("Expected List node, got {:?}", ast_nodes[0]),
        }
    }
}
