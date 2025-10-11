use crate::tokenizer::tokens::{Token, TokenType};
use crate::tokenizer::verbatim_scanner::{VerbatimBlock, VerbatimScanner};
use regex::Regex;

pub struct Lexer {
    text: String,
    tokens: Vec<Token>,
    current_line: usize,
    current_col: usize,
    indent_stack: Vec<usize>,
    verbatim_blocks: Vec<VerbatimBlock>,

    // Pre-compiled regex patterns for better performance
    pragma_re: Regex,
    empty_pragma_re: Regex,
    verbatim_start_re: Regex,
    verbatim_end_re: Regex,
}

impl Lexer {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            tokens: Vec::new(),
            current_line: 1,
            current_col: 1,
            indent_stack: vec![0],
            verbatim_blocks: Vec::new(),
            pragma_re: Regex::new(r"^::\s*(.+?)\s*::(?:\s+(.*))?$").unwrap(),
            empty_pragma_re: Regex::new(r"^::\s*::$").unwrap(),
            verbatim_start_re: Regex::new(r"^(.*?)\s*:\s*$").unwrap(),
            verbatim_end_re: Regex::new(r"^\(([^)]*)\)\s*$").unwrap(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        // Handle empty input
        if self.text.is_empty() {
            self.add_token(TokenType::Eof, Some("".to_string()));
            return self.tokens.clone();
        }

        // Pass 0: Scan for verbatim blocks
        let scanner = VerbatimScanner::new();
        self.verbatim_blocks = scanner.scan(&self.text);

        // Collect lines to avoid borrowing issues
        let lines: Vec<String> = self.text.lines().map(|s| s.to_string()).collect();

        for (line_idx, line) in lines.iter().enumerate() {
            self.current_line = line_idx + 1;
            self.current_col = 1;

            // Calculate current indentation
            let expanded_line = line.replace('\t', "    ");
            let stripped_line = expanded_line.trim_start();
            let indentation = expanded_line.len() - stripped_line.len();

            // Update column position to account for indentation
            self.current_col = indentation + 1;

            // Handle blank lines (only whitespace)
            if stripped_line.is_empty() {
                self.add_token(TokenType::BlankLine, Some("\n".to_string()));
                continue;
            }

            // Handle indentation changes
            if indentation > self.indent_stack[self.indent_stack.len() - 1] {
                self.indent_stack.push(indentation);
                self.add_token(TokenType::Indent, Some("".to_string()));
            } else {
                // Emit one DEDENT token for each level we're backing out
                while indentation < self.indent_stack[self.indent_stack.len() - 1]
                    && self.indent_stack.len() > 1
                {
                    self.indent_stack.pop();
                    self.add_token(TokenType::Dedent, Some("".to_string()));
                }
            }

            // Process line content
            let scanner = VerbatimScanner::new();
            if scanner.is_verbatim_content(self.current_line, &self.verbatim_blocks) {
                // It's verbatim content - emit the entire line as-is
                self.add_token(TokenType::VerbatimContent, Some(expanded_line));
            } else {
                // Normal processing
                self.process_line_content(stripped_line, line, indentation);
            }

            // Add newline token - should have empty value
            self.add_token(TokenType::Newline, Some("".to_string()));
        }

        // Ensure any remaining indents are closed at the end of the file
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.add_token(TokenType::Dedent, None);
        }

        self.add_token(TokenType::Eof, Some("".to_string()));
        self.tokens.clone()
    }

    fn process_line_content(&mut self, line: &str, full_line: &str, indentation: usize) {
        // Check for pragma/annotation (:: label ::)
        if let Some(_captures) = self.empty_pragma_re.captures(line) {
            self.add_token(TokenType::PragmaMarker, Some("::".to_string()));
            self.add_token(TokenType::PragmaMarker, Some("::".to_string()));
            return;
        }

        if let Some(captures) = self.pragma_re.captures(line) {
            self.add_token(TokenType::PragmaMarker, Some("::".to_string()));
            let label_part = captures.get(1).unwrap().as_str();
            self.process_pragma_label(label_part);
            self.add_token(TokenType::PragmaMarker, Some("::".to_string()));
            if let Some(value_part) = captures.get(2) {
                self.add_token(TokenType::Text, Some(value_part.as_str().to_string()));
            }
            return;
        }

        // Check for definition (line ending with :: or :: followed by annotation)
        let stripped_line = line.trim_end();
        let definition_re = Regex::new(r"^(.+?)\s*::\s*(\(.+\))?\s*$").unwrap();
        if let Some(captures) = definition_re.captures(stripped_line) {
            let term = captures.get(1).unwrap().as_str().trim();
            let annotation = captures.get(2).map(|m| m.as_str());

            if !term.is_empty() {
                self.process_text_with_inline_formatting(term);
            }
            self.add_token(TokenType::DefinitionMarker, Some("::".to_string()));
            if let Some(annotation) = annotation {
                self.add_token(TokenType::Text, Some(format!(" {}", annotation)));
            }
            return;
        }

        // Check for verbatim block start (line ending with : but not ::)
        if let Some(captures) = self.verbatim_start_re.captures(line) {
            let content = captures.get(1).unwrap().as_str().trim();
            // Make sure it doesn't end with :: (pragma marker)
            if !content.ends_with(':') {
                if !content.is_empty() {
                    self.add_token(TokenType::Text, Some(content.to_string()));
                }
                self.add_token(TokenType::VerbatimStart, Some(":".to_string()));
                return;
            }
        }

        // Check for verbatim block end: (label) or ()
        if let Some(captures) = self.verbatim_end_re.captures(line) {
            self.add_token(TokenType::VerbatimEnd, Some("(".to_string()));
            let label = captures.get(1).unwrap().as_str();
            if !label.is_empty() {
                self.process_pragma_label(label);
            }
            self.add_token(TokenType::VerbatimEnd, Some(")".to_string()));
            return;
        }

        // Check for dash list marker
        let dash_re = Regex::new(r"^(\s*)(-\s+)").unwrap();
        if let Some(captures) = dash_re.captures(full_line) {
            let indentation_str = captures.get(1).unwrap().as_str();
            let dash_and_space = captures.get(2).unwrap().as_str();

            // Only match if indentation matches expected level
            if indentation_str.replace('\t', "    ").len() == indentation {
                self.add_token(TokenType::Dash, Some(dash_and_space.to_string()));
                let rest_start = indentation_str.len() + dash_and_space.len();
                if rest_start < full_line.len() {
                    let rest = &full_line[rest_start..];
                    if !rest.is_empty() {
                        self.process_text_with_inline_formatting(rest);
                    }
                }
                return;
            }
        }

        // Check for sequence markers (1., a), etc.)
        let seq_re = Regex::new(r"^(\s*)(\d+(?:\.\d+)*\.|[a-zA-Z]\)|[a-zA-Z]\.)(\s+)").unwrap();
        if let Some(captures) = seq_re.captures(full_line) {
            let indentation_str = captures.get(1).unwrap().as_str();
            let marker = captures.get(2).unwrap().as_str();
            let whitespace = captures.get(3).unwrap().as_str();

            // Only match if indentation matches expected level
            if indentation_str.replace('\t', "    ").len() == indentation {
                // Capture marker with its following whitespace
                self.add_token(
                    TokenType::SequenceMarker,
                    Some(format!("{}{}", marker, whitespace)),
                );
                let rest_start = indentation_str.len() + marker.len() + whitespace.len();
                if rest_start < full_line.len() {
                    let rest = &full_line[rest_start..];
                    if !rest.is_empty() {
                        self.process_text_with_inline_formatting(rest);
                    }
                }
                return;
            }
        }

        // Default: process as text with potential inline formatting
        self.process_text_with_inline_formatting(line);
    }

    fn process_pragma_label(&mut self, label: &str) {
        // Check if label contains parameters (key:value)
        if label.contains(':') {
            let parts: Vec<&str> = label.splitn(2, ':').collect();
            self.add_token(TokenType::Identifier, Some(parts[0].to_string()));
            self.add_token(TokenType::Colon, Some(":".to_string()));
            self.process_parameters(parts[1]);
        } else {
            self.add_token(TokenType::Identifier, Some(label.to_string()));
        }
    }

    fn process_parameters(&mut self, params_str: &str) {
        let params: Vec<&str> = params_str.split(',').collect();
        for (i, param) in params.iter().enumerate() {
            let param = param.trim();
            if param.contains('=') {
                let parts: Vec<&str> = param.splitn(2, '=').collect();
                let key = parts[0].trim();
                let value = parts[1].trim();

                self.add_token(TokenType::Identifier, Some(key.to_string()));
                self.add_token(TokenType::Equals, Some("=".to_string()));

                // Check if value is quoted
                if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                    self.add_token(TokenType::String, Some(value.to_string()));
                } else {
                    self.add_token(TokenType::Identifier, Some(value.to_string()));
                }
            } else {
                self.add_token(TokenType::Identifier, Some(param.to_string()));
            }

            if i < params.len() - 1 {
                self.add_token(TokenType::Comma, Some(",".to_string()));
            }
        }
    }

    fn process_text_with_inline_formatting(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let mut i = 0;
        let mut current_text = String::new();
        let chars: Vec<char> = text.chars().collect();

        while i < chars.len() {
            let ch = chars[i];

            // Check for reference/footnote markers [...]
            if ch == '[' {
                // Emit any accumulated text first
                if !current_text.is_empty() {
                    self.add_token(TokenType::Text, Some(current_text.clone()));
                    current_text.clear();
                }

                // Find the matching closing bracket
                let mut bracket_count = 1;
                let mut j = i + 1;
                let mut ref_content = String::new();

                while j < chars.len() && bracket_count > 0 {
                    if chars[j] == '[' {
                        bracket_count += 1;
                    } else if chars[j] == ']' {
                        bracket_count -= 1;
                    }
                    if bracket_count > 0 {
                        ref_content.push(chars[j]);
                    }
                    j += 1;
                }

                if bracket_count == 0 {
                    // Determine reference type
                    if ref_content.starts_with('@') {
                        // Citation [@key] or [@key, p. 45]
                        self.add_token(TokenType::Citation, Some(format!("[{}]", ref_content)));
                    } else if ref_content.chars().all(|c| c.is_ascii_digit()) {
                        self.add_token(
                            TokenType::FootnoteNumber,
                            Some(format!("[{}]", ref_content)),
                        );
                    } else if ref_content.starts_with('#') && ref_content.len() > 1 {
                        // Session reference like #3.4
                        self.add_token(
                            TokenType::SessionNumber,
                            Some(format!("[{}]", ref_content)),
                        );
                    } else {
                        self.add_token(TokenType::RefMarker, Some(format!("[{}]", ref_content)));
                    }
                    i = j;
                    continue;
                } else {
                    // Unclosed bracket, treat as literal
                    current_text.push(ch);
                    i += 1;
                    continue;
                }
            }

            // Check for inline formatting markers (*, _, `, #)
            if matches!(ch, '*' | '_' | '`' | '#') {
                // Check if this could be a valid formatting marker
                if i + 1 < chars.len() && chars[i + 1] != ' ' {
                    // Look for the closing marker
                    let mut j = i + 1;
                    while j < chars.len() && chars[j] != ch {
                        j += 1;
                    }

                    if j < chars.len() && j > i + 1 {
                        // Found a matching closing marker
                        // Check that it's not followed by alphanumeric
                        if j == chars.len() - 1 || !chars[j + 1].is_alphanumeric() {
                            // Emit accumulated text
                            if !current_text.is_empty() {
                                self.add_token(TokenType::Text, Some(current_text.clone()));
                                current_text.clear();
                            }

                            // Emit formatting tokens
                            let marker_type = match ch {
                                '*' => TokenType::StrongMarker,
                                '_' => TokenType::EmphasisMarker,
                                '`' => TokenType::CodeMarker,
                                '#' => TokenType::MathMarker,
                                _ => unreachable!(),
                            };

                            self.add_token(marker_type.clone(), Some(ch.to_string()));
                            let content: String = chars[(i + 1)..j].iter().collect();
                            if !content.is_empty() {
                                self.add_token(TokenType::Text, Some(content));
                            }
                            self.add_token(marker_type, Some(ch.to_string()));

                            i = j + 1;
                            continue;
                        }
                    }
                }
            }

            // Default: accumulate as text
            current_text.push(ch);
            i += 1;
        }

        // Emit any remaining text
        if !current_text.is_empty() {
            self.add_token(TokenType::Text, Some(current_text));
        }
    }

    fn add_token(&mut self, token_type: TokenType, value: Option<String>) {
        self.tokens.push(Token::new(
            token_type.clone(),
            value.clone(),
            self.current_line,
            self.current_col,
        ));

        // Update column position based on the token value
        if let Some(ref val) = value {
            self.current_col += val.len();
        } else if !matches!(token_type, TokenType::Indent | TokenType::Dedent) {
            // For tokens like NEWLINE, advance by 1
            // But INDENT/DEDENT don't consume any columns
            self.current_col += 1;
        }
    }
}
