//! Verbatim Block Scanner
//!
//! The verbatim scanner is a **pre-tokenization** step that identifies verbatim blocks from raw text
//! and marks their boundaries before any TXXT parsing begins.
//!
//! ## Core Purpose
//!
//! Verbatim blocks contain non-TXXT content that must be preserved exactly. Since this content would
//! produce gibberish if parsed as TXXT (especially indent tokens), we must identify these blocks first
//! and mark them as opaque content.
//!
//! ## Verbatim Block Syntax
//!
//! Verbatim blocks have three components:
//! 1. **Title line**: Optional text followed by a single `:` at end of line
//! 2. **Content lines**: Everything between title and terminator (preserved exactly)
//! 3. **Terminator line**: `:: label` or `:: label:params` with optional parameters
//!
//! ### Normal Verbatim Block
//! ```txxt
//! title:
//!     Content starts +1 indented
//!     Another line
//!
//!     Blank lines are allowed
//! :: identifier
//! ```
//!
//! ### Stretched Verbatim Block
//! ```txxt
//! title:
//! Content starts at column 0
//! Another line at column 0
//!
//! Blank lines are allowed
//! :: identifier
//! ```
//!
//! ### With Parameters
//! ```txxt
//! title:
//!     Content here
//! :: label:key1=value1,key2=value2
//! ```
//!
//! ## Scanner State Machine
//!
//! The scanner uses a state machine to identify verbatim blocks:
//!
//! **States:**
//! - `ScanningNormal` - Looking for potential verbatim starts
//! - `FoundPotentialStart` - Just found a line ending with `:`, checking next line
//! - `InVerbatimNormal` - Inside normal verbatim (+1 indented content)
//! - `InVerbatimStretched` - Inside stretched verbatim (column 0 content)
//!
//! ## Algorithm Steps
//!
//! ### 1. Scan for Potential Verbatim Start
//! - Look for lines ending with single `:` (not `::`)
//! - Record the **indentation level** of this title line
//! - Transition to `FoundPotentialStart`
//!
//! ### 2. Validate Verbatim Start (Next Line Analysis)
//! After finding potential start, examine the very next non-blank line:
//!
//! **If next line is:**
//! - **Blank**: Continue scanning (could be either type)
//! - **At column 0**: Stretched verbatim → `InVerbatimStretched`
//! - **At title_indent + 1**: Normal verbatim → `InVerbatimNormal`
//! - **Anything else**: False alarm, return to `ScanningNormal`
//!
//! ### 3. Content Scanning (Normal Verbatim)
//! While in `InVerbatimNormal`:
//! - **Expected**: Content at `title_indent + 1` or deeper
//! - **Blank lines**: Allowed, continue
//! - **Content at wrong indent**: Error or end of block
//! - **Terminator found**: Validate and end block
//!
//! ### 4. Content Scanning (Stretched Verbatim)
//! While in `InVerbatimStretched`:
//! - **Expected**: Content at column 0
//! - **Blank lines**: Allowed, continue
//! - **Content at non-zero column**: Must be terminator at title indent
//! - **Terminator found**: Validate and end block
//!
//! ### 5. Terminator Validation
//! Valid terminator must:
//! - Be at **exact same indentation** as title line
//! - Match pattern: `:: identifier` or `:: identifier:params`
//! - Have proper parameter syntax if present
//!
//! ## Error Conditions
//! - **No terminator found**: Document ends while in verbatim mode
//! - **Invalid terminator syntax**: Malformed `:: label` line
//! - **Wrong terminator indent**: Not aligned with title
//! - **Content at wrong indent**: Breaks verbatim rules
//!
//! ## Critical Rules
//! 1. **Annotation lines `:: label ::`** are NEVER verbatim starts
//! 2. **Definition lines ending `::` are NEVER verbatim starts
//! 3. **Indentation must be exact** - no fuzzy matching
//! 4. **Content type determined by first non-blank line** after title
//! 5. **Terminator indent must match title indent exactly**

use crate::cst::{Position, ScannerToken, SourceSpan, WallType};
use crate::lexer::elements::components::parameters::{parse_parameters, ParameterLexer};
use regex::Regex;

/// Standard indentation level in spaces
const INDENT_SIZE: usize = 4;

// THE src/tokenizer/verbatim_scanner.rs HAS THE RULES IN THE DOCS DO NOT FRAUD, LIE NOR MAKE UP RULES

/// Type of verbatim block based on content indentation
#[derive(Debug, Clone, PartialEq)]
pub enum VerbatimType {
    /// Content is indented +1 relative to title
    Normal,
    /// Content starts at column 0
    Stretched,
    /// No content - terminator immediately follows title
    Empty,
}

/// A detected verbatim block with line boundaries
#[derive(Debug, Clone, PartialEq)]
pub struct VerbatimBlock {
    /// Line number where the verbatim block starts (title line, 1-based)
    pub block_start: usize,
    /// Line number where the verbatim block ends (terminator line, 1-based)
    pub block_end: usize,
    /// Type of verbatim block
    pub block_type: VerbatimType,
    /// Indentation level of the title line
    pub title_indent: usize,
    /// First line of verbatim content (1-based, inclusive) - None for empty blocks
    pub content_start: Option<usize>,
    /// Last line of verbatim content (1-based, inclusive) - None for empty blocks
    pub content_end: Option<usize>,
}

/// State of the verbatim scanner
#[derive(Debug, Clone, PartialEq)]
enum ScanState {
    /// Scanning for potential verbatim starts
    ScanningNormal,
    /// Found potential start, validating next line
    FoundPotentialStart {
        title_line: usize,
        title_indent: usize,
    },
    /// Inside normal verbatim block (+1 indented content)
    InVerbatimNormal {
        title_line: usize,
        title_indent: usize,
        content_start: usize,
        expected_indent: usize,
    },
    /// Inside stretched verbatim block (column 0 content)
    InVerbatimStretched {
        title_line: usize,
        title_indent: usize,
        content_start: usize,
    },
}

/// Pre-tokenization verbatim block scanner
pub struct VerbatimScanner {
    /// Regex for detecting potential verbatim start (line ending with single :)
    verbatim_start_re: Regex,
    /// Regex for detecting verbatim terminator (label with optional params)
    verbatim_end_re: Regex,
    /// Regex for detecting annotation lines (never verbatim starts)
    annotation_re: Regex,
    /// Regex for detecting definition lines (never verbatim starts)
    definition_re: Regex,
}

impl Default for VerbatimScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl VerbatimScanner {
    /// Create a new verbatim scanner
    pub fn new() -> Self {
        Self {
            // Match line ending with single : (not ::)
            verbatim_start_re: Regex::new(r"^(.*):\s*$").unwrap(),
            // Match terminator: :: identifier or :: identifier:params
            verbatim_end_re: Regex::new(r"^\s*::\s+([a-zA-Z_][a-zA-Z0-9._-]*(?::[^:\s].*)?)\s*$")
                .unwrap(),
            // Match annotation lines :: label ::
            annotation_re: Regex::new(r"^.*::\s*.*::\s*.*$").unwrap(),
            // Match definition lines ending with ::
            definition_re: Regex::new(r"^.*::\s*$").unwrap(),
        }
    }

    /// Scan text for verbatim blocks, returning a list of detected blocks
    pub fn scan(&self, text: &str) -> Vec<VerbatimBlock> {
        let mut blocks = Vec::new();
        let mut state = ScanState::ScanningNormal;
        let lines: Vec<&str> = text.lines().collect();
        let mut line_idx = 0;

        while line_idx < lines.len() {
            let line_num = line_idx + 1; // 1-based line numbers
            let line = lines[line_idx];

            let (new_state, next_line_idx) =
                self.process_line_with_backtrack(&mut blocks, state, line_num, line, &lines);

            state = new_state;
            line_idx = next_line_idx;
        }

        // Handle end of document
        if let Err(error) = self.finalize_scan(&mut blocks, state, lines.len()) {
            eprintln!("Verbatim scanner error: {}", error);
        }

        blocks
    }

    /// Process a single line in the state machine with backtracking support
    fn process_line_with_backtrack(
        &self,
        blocks: &mut Vec<VerbatimBlock>,
        state: ScanState,
        line_num: usize,
        line: &str,
        _all_lines: &[&str],
    ) -> (ScanState, usize) {
        let next_state = self.process_line(blocks, state.clone(), line_num, line, _all_lines);

        // Check if we need to backtrack
        match (&state, &next_state) {
            // If we were in verbatim mode and now we're scanning normal, we may need to backtrack
            (ScanState::InVerbatimNormal { title_line, .. }, ScanState::ScanningNormal)
            | (ScanState::InVerbatimStretched { title_line, .. }, ScanState::ScanningNormal) => {
                // Backtrack to line after the failed title line
                (next_state, *title_line) // title_line is 1-based, so this points to line after title
            }
            _ => {
                // Normal progression to next line
                (next_state, line_num)
            }
        }
    }

    /// Process a single line in the state machine
    fn process_line(
        &self,
        blocks: &mut Vec<VerbatimBlock>,
        state: ScanState,
        line_num: usize,
        line: &str,
        _all_lines: &[&str],
    ) -> ScanState {
        match state {
            ScanState::ScanningNormal => self.check_for_verbatim_start(line_num, line),

            ScanState::FoundPotentialStart {
                title_line,
                title_indent,
            } => self.validate_verbatim_start(blocks, title_line, title_indent, line_num, line),

            ScanState::InVerbatimNormal {
                title_line,
                title_indent,
                content_start,
                expected_indent,
            } => self.process_normal_verbatim_line(
                blocks,
                title_line,
                title_indent,
                content_start,
                expected_indent,
                line_num,
                line,
            ),

            ScanState::InVerbatimStretched {
                title_line,
                title_indent,
                content_start,
            } => self.process_stretched_verbatim_line(
                blocks,
                title_line,
                title_indent,
                content_start,
                line_num,
                line,
            ),
        }
    }

    /// Check if a line could be a verbatim start
    fn check_for_verbatim_start(&self, line_num: usize, line: &str) -> ScanState {
        // Skip annotation lines - they are never verbatim starts
        if self.annotation_re.is_match(line) {
            return ScanState::ScanningNormal;
        }

        // Skip definition lines ending with :: - they are never verbatim starts
        if self.definition_re.is_match(line) {
            return ScanState::ScanningNormal;
        }

        // Check for potential verbatim start (line ending with single :)
        if let Some(captures) = self.verbatim_start_re.captures(line) {
            let prefix = captures.get(1).unwrap().as_str();

            // Make sure it doesn't end with :: (that would be a definition)
            if !prefix.ends_with(':') {
                let title_indent = self.calculate_indentation(line);
                return ScanState::FoundPotentialStart {
                    title_line: line_num,
                    title_indent,
                };
            }
        }

        ScanState::ScanningNormal
    }

    /// Validate that the next line confirms this is a verbatim block
    fn validate_verbatim_start(
        &self,
        blocks: &mut Vec<VerbatimBlock>,
        title_line: usize,
        title_indent: usize,
        line_num: usize,
        line: &str,
    ) -> ScanState {
        // If this is a blank line, continue waiting for content
        if line.trim().is_empty() {
            return ScanState::FoundPotentialStart {
                title_line,
                title_indent,
            };
        }

        // Check if this line is an annotation - if so, this is NOT a verbatim block
        if self.annotation_re.is_match(line) {
            return ScanState::ScanningNormal;
        }

        let line_indent = self.calculate_indentation(line);

        // Check for terminator immediately after title (empty verbatim block)
        if self.is_valid_terminator(line, title_indent) {
            // This is an empty verbatim block - add it and continue scanning
            blocks.push(VerbatimBlock {
                block_start: title_line,
                block_end: line_num,
                block_type: VerbatimType::Empty,
                title_indent,
                content_start: None,
                content_end: None,
            });
            return ScanState::ScanningNormal;
        }

        // Determine verbatim type based on content indentation
        if line_indent == 0 {
            // Stretched verbatim - content at column 0
            ScanState::InVerbatimStretched {
                title_line,
                title_indent,
                content_start: line_num,
            }
        } else if line_indent == title_indent + INDENT_SIZE {
            // Normal verbatim - content at +1 indentation level from title
            ScanState::InVerbatimNormal {
                title_line,
                title_indent,
                content_start: line_num,
                expected_indent: line_indent,
            }
        } else {
            // Not a valid verbatim block - wrong indentation
            ScanState::ScanningNormal
        }
    }

    /// Process a line while in normal verbatim mode
    #[allow(clippy::too_many_arguments)]
    fn process_normal_verbatim_line(
        &self,
        blocks: &mut Vec<VerbatimBlock>,
        title_line: usize,
        title_indent: usize,
        content_start: usize,
        expected_indent: usize,
        line_num: usize,
        line: &str,
    ) -> ScanState {
        // Allow blank lines
        if line.trim().is_empty() {
            return ScanState::InVerbatimNormal {
                title_line,
                title_indent,
                content_start,
                expected_indent,
            };
        }

        // Check for valid terminator
        if self.is_valid_terminator(line, title_indent) {
            // End of verbatim block
            blocks.push(VerbatimBlock {
                block_start: title_line,
                block_end: line_num,
                block_type: VerbatimType::Normal,
                title_indent,
                content_start: Some(content_start),
                content_end: Some(line_num - 1),
            });
            return ScanState::ScanningNormal;
        }

        let line_indent = self.calculate_indentation(line);

        // Content must be at expected indent (title + INDENT_SIZE) or deeper
        if line_indent >= title_indent + INDENT_SIZE {
            ScanState::InVerbatimNormal {
                title_line,
                title_indent,
                content_start,
                expected_indent,
            }
        } else {
            // Wrong indentation - abandon verbatim block
            ScanState::ScanningNormal
        }
    }

    /// Process a line while in stretched verbatim mode
    fn process_stretched_verbatim_line(
        &self,
        blocks: &mut Vec<VerbatimBlock>,
        title_line: usize,
        title_indent: usize,
        content_start: usize,
        line_num: usize,
        line: &str,
    ) -> ScanState {
        // Allow blank lines
        if line.trim().is_empty() {
            return ScanState::InVerbatimStretched {
                title_line,
                title_indent,
                content_start,
            };
        }

        let line_indent = self.calculate_indentation(line);

        // Check for terminator first (must be at title indent)
        if self.is_valid_terminator(line, title_indent) {
            blocks.push(VerbatimBlock {
                block_start: title_line,
                block_end: line_num,
                block_type: VerbatimType::Stretched,
                title_indent,
                content_start: Some(content_start),
                content_end: Some(line_num - 1),
            });
            return ScanState::ScanningNormal;
        }

        // Content should be at column 0
        if line_indent == 0 {
            return ScanState::InVerbatimStretched {
                title_line,
                title_indent,
                content_start,
            };
        }

        // Invalid - expected terminator or content at column 0
        ScanState::ScanningNormal
    }

    /// Check if a line is a valid verbatim terminator at the expected indentation
    fn is_valid_terminator(&self, line: &str, expected_indent: usize) -> bool {
        let line_indent = self.calculate_indentation(line);

        // Terminator must be at exact same indentation as title
        if line_indent != expected_indent {
            return false;
        }

        // Must match terminator pattern
        self.verbatim_end_re.is_match(line)
    }

    /// Calculate indentation level of a line (number of leading spaces, tabs = 4 spaces)
    fn calculate_indentation(&self, line: &str) -> usize {
        let mut indent = 0;
        for ch in line.chars() {
            match ch {
                ' ' => indent += 1,
                '\t' => indent += 4,
                _ => break,
            }
        }
        indent
    }

    /// Handle end of document - check for unterminated verbatim blocks
    fn finalize_scan(
        &self,
        _blocks: &mut Vec<VerbatimBlock>,
        state: ScanState,
        _total_lines: usize,
    ) -> Result<(), String> {
        match state {
            ScanState::ScanningNormal | ScanState::FoundPotentialStart { .. } => {
                // No active verbatim block, all good
                Ok(())
            }
            ScanState::InVerbatimNormal { title_line, .. }
            | ScanState::InVerbatimStretched { title_line, .. } => Err(format!(
                "Unterminated verbatim block starting at line {}",
                title_line
            )),
        }
    }

    /// Check if a line number is within verbatim content
    pub fn is_verbatim_content(&self, line_num: usize, blocks: &[VerbatimBlock]) -> bool {
        blocks.iter().any(|block| {
            if let (Some(start), Some(end)) = (block.content_start, block.content_end) {
                line_num >= start && line_num <= end
            } else {
                false // Empty blocks have no content
            }
        })
    }
}

/// Trait for verbatim block tokenization
pub trait VerbatimLexer: ParameterLexer + Sized {
    /// Get current row (line number)
    fn row(&self) -> usize;

    /// Get current column
    fn column(&self) -> usize;

    /// Get absolute position in input
    fn get_absolute_position(&self) -> usize;

    /// Read verbatim block if current position matches a verbatim block start
    fn read_verbatim_block(
        &mut self,
        verbatim_blocks: &[VerbatimBlock],
    ) -> Option<Vec<ScannerToken>> {
        let current_line = self.row();
        let current_char_pos = self.get_absolute_position();

        // Find a verbatim block that starts at this position
        for block in verbatim_blocks {
            if self.is_at_verbatim_block_start(block, current_line, current_char_pos) {
                return Some(self.tokenize_verbatim_block(block));
            }
        }

        None
    }

    /// Check if current position is at the start of the given verbatim block
    fn is_at_verbatim_block_start(
        &self,
        block: &VerbatimBlock,
        current_line: usize,
        _current_char_pos: usize,
    ) -> bool {
        // Check if we're at the correct line for block start (1-based to 0-based conversion)
        (block.block_start - 1) == current_line && self.is_at_line_start_for_verbatim(block)
    }

    /// Check if we're at the start of a line that should be part of a verbatim block
    fn is_at_line_start_for_verbatim(&self, _block: &VerbatimBlock) -> bool {
        // For now, just check if we're at the start of a line
        self.column() == 0
    }

    /// Tokenize a verbatim block into VerbatimTitle, IndentationWall, and IgnoreTextSpan tokens
    fn tokenize_verbatim_block(&mut self, block: &VerbatimBlock) -> Vec<ScannerToken> {
        let mut tokens = Vec::new();

        // Create VerbatimTitle token for the title line
        let title_start_pos = self.current_position();

        // Advance through the title line to get its content (excluding the trailing colon)
        let mut title_content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            if ch == ':' {
                // Don't include the colon - it's a structural marker, not content
                self.advance();
                break;
            }
            title_content.push(ch);
            self.advance();
        }

        // Advance past the newline
        if let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                self.advance();
                if ch == '\r' && self.peek() == Some('\n') {
                    self.advance(); // Handle CRLF
                }
            }
        }

        // Calculate the wall position and type first
        let (wall_level, wall_type) = match block.block_type {
            VerbatimType::Normal => (
                block.title_indent + INDENT_SIZE,
                WallType::InFlow(block.title_indent),
            ),
            VerbatimType::Stretched => (0, WallType::Stretched), // Content starts at column 0
            VerbatimType::Empty => (0, WallType::Stretched), // No content, but use stretched for consistency
        };

        // Emit IndentationWall token first
        tokens.push(ScannerToken::IndentationWall {
            level: wall_level,
            wall_type: wall_type.clone(),
            span: SourceSpan {
                start: title_start_pos,
                end: title_start_pos,
            },
        });

        // Create VerbatimTitle token
        tokens.push(ScannerToken::VerbatimTitle {
            content: title_content.trim_start().to_string(),
            span: SourceSpan {
                start: title_start_pos,
                end: self.current_position(),
            },
        });

        // Process content lines if they exist
        if let (Some(_content_start), Some(_content_end)) = (block.content_start, block.content_end)
        {
            let mut current_line = self.row();
            let mut all_content = String::new();
            let mut first_line_start_pos = None;
            let mut last_line_end_pos = None;

            while current_line < (block.block_end - 1) {
                // Convert 1-based to 0-based
                let line_start_pos = self.current_position();
                if first_line_start_pos.is_none() {
                    first_line_start_pos = Some(line_start_pos);
                }
                let mut line_content = String::new();

                // Read the entire line
                while let Some(ch) = self.peek() {
                    if ch == '\n' || ch == '\r' {
                        // Add the newline to content
                        line_content.push(ch);
                        self.advance();
                        if ch == '\r' && self.peek() == Some('\n') {
                            line_content.push('\n');
                            self.advance(); // Handle CRLF
                        }
                        break;
                    } else {
                        line_content.push(ch);
                        self.advance();
                    }
                }

                // Strip wall indentation from content for IgnoreTextSpan
                let mut content_without_wall = String::new();
                let mut col = 0;
                for ch in line_content.chars() {
                    if col >= wall_level {
                        content_without_wall.push(ch);
                    }
                    col += if ch == '\t' { 4 } else { 1 };
                }

                // Add to all content
                all_content.push_str(&content_without_wall);
                last_line_end_pos = Some(self.current_position());

                current_line = self.row();
            }

            // Remove trailing newline from all content
            if all_content.ends_with('\n') {
                all_content.pop();
                if all_content.ends_with('\r') {
                    all_content.pop();
                }
            }

            if !all_content.is_empty() {
                // Create wall position span (from start of first line to wall position)
                let _wall_span = SourceSpan {
                    start: first_line_start_pos.unwrap(),
                    end: Position {
                        row: first_line_start_pos.unwrap().row,
                        column: first_line_start_pos.unwrap().column + wall_level,
                    },
                };

                // Create content span (from wall position to end of last line)
                let content_span = SourceSpan {
                    start: Position {
                        row: first_line_start_pos.unwrap().row,
                        column: first_line_start_pos.unwrap().column + wall_level,
                    },
                    end: last_line_end_pos.unwrap(),
                };

                // Emit IgnoreTextSpan token with all content without wall indentation
                tokens.push(ScannerToken::IgnoreTextSpan {
                    content: all_content,
                    span: content_span,
                });
            }
        }

        // Parse the terminator line to extract label and parameters
        let terminator_start_pos = self.current_position();
        let mut terminator_content = String::new();

        // Read the entire terminator line content
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            terminator_content.push(ch);
            self.advance();
        }

        // Extract just the label+params portion (without :: prefix)
        if !terminator_content.trim().is_empty() {
            // Use the same regex pattern as the verbatim scanner
            let verbatim_end_re =
                Regex::new(r"^\s*::\s+([a-zA-Z_][a-zA-Z0-9._-]*(?::[^:\s].*)?)\s*$").unwrap();

            if let Some(captures) = verbatim_end_re.captures(&terminator_content) {
                if let Some(label_and_params) = captures.get(1) {
                    let label_and_params_str = label_and_params.as_str();

                    // Split label from parameters at the first colon
                    if let Some(colon_pos) = label_and_params_str.find(':') {
                        // There are parameters - split them
                        let label = &label_and_params_str[..colon_pos];
                        let params_str = &label_and_params_str[colon_pos + 1..];

                        // Add the clean verbatim label
                        tokens.push(ScannerToken::VerbatimLabel {
                            content: label.to_string(),
                            span: SourceSpan {
                                start: terminator_start_pos,
                                end: self.current_position(),
                            },
                        });

                        // Parse and add individual parameter tokens
                        let mut param_tokens = parse_parameters(self, params_str);
                        tokens.append(&mut param_tokens);
                    } else {
                        // No parameters - just the label
                        tokens.push(ScannerToken::VerbatimLabel {
                            content: label_and_params_str.to_string(),
                            span: SourceSpan {
                                start: terminator_start_pos,
                                end: self.current_position(),
                            },
                        });
                    }
                }
            } else {
                // Fallback: if regex doesn't match, use the full content (shouldn't happen)
                tokens.push(ScannerToken::VerbatimLabel {
                    content: terminator_content,
                    span: SourceSpan {
                        start: terminator_start_pos,
                        end: self.current_position(),
                    },
                });
            }
        }

        // Don't advance past the newline at end of terminator
        // The newline should be tokenized separately, potentially as part of a BlankLine

        tokens
    }
}
