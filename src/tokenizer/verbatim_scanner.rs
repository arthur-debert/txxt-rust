use regex::Regex;

/// Represents a verbatim block identified during pre-scanning
///
/// Verbatim blocks have the structure:
/// ```text
/// title:
///     content line 1
///     content line 2
/// (label)
/// ```
///
/// This struct identifies the line boundaries so the tokenizer can treat
/// content lines differently from normal TXXT lines.
#[derive(Debug, Clone, PartialEq)]
pub struct VerbatimBlock {
    pub start_line: usize, // 1-indexed line number where verbatim starts (title line)
    pub end_line: usize,   // 1-indexed line number where verbatim ends (label line)
    pub start_indent: usize, // Indentation level of the opening line
}

pub struct VerbatimScanner {
    verbatim_start_re: Regex,
    verbatim_end_re: Regex,
}

impl VerbatimScanner {
    pub fn new() -> Self {
        Self {
            verbatim_start_re: Regex::new(r"^(\s*)(.*?)\s*:\s*$").unwrap(),
            verbatim_end_re: Regex::new(r"^(\s*)\(([^)]*)\)\s*$").unwrap(),
        }
    }

    /// Pre-scan text to identify verbatim block boundaries
    ///
    /// This is the critical first pass that identifies where verbatim blocks
    /// start and end BEFORE tokenization begins. It returns line ranges
    /// like [[3,6], [10,15]] that tell the tokenizer which lines contain
    /// verbatim content that should be preserved exactly.
    ///
    /// Verbatim blocks follow this pattern:
    /// - Start: Line ending with ":" (but not "::")
    /// - Content: Lines indented relative to start line
    /// - End: Line at same indent as start, matching "(<label>)" pattern
    pub fn scan(&self, text: &str) -> Vec<VerbatimBlock> {
        let lines: Vec<&str> = text.lines().collect();
        let mut blocks = Vec::new();
        let mut current_block: Option<VerbatimBlock> = None;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Calculate indentation
            let expanded_line = line.replace('\t', "    ");
            let stripped = expanded_line.trim_start();
            let indent = expanded_line.len() - stripped.len();

            if current_block.is_none() {
                // Not in a verbatim block - check for start pattern
                // Must end with ":" but NOT with "::" (that's a definition)
                if self.verbatim_start_re.is_match(line) && !line.trim_end().ends_with("::") {
                    current_block = Some(VerbatimBlock {
                        start_line: line_num,
                        end_line: line_num, // Will be updated when we find the end
                        start_indent: indent,
                    });
                }
            } else {
                // In a verbatim block - check for end pattern
                if let Some(ref mut block) = current_block {
                    // End must be at same indentation and match "(<label>)" pattern
                    if indent == block.start_indent && self.verbatim_end_re.is_match(line) {
                        // Found the matching end
                        block.end_line = line_num;
                        blocks.push(block.clone());
                        current_block = None;
                    }
                }
            }
        }

        // If we have an unclosed verbatim block, close it at EOF
        if let Some(mut block) = current_block {
            block.end_line = lines.len();
            blocks.push(block);
        }

        blocks
    }

    /// Check if a specific line number is verbatim content
    ///
    /// Returns true if the line is INSIDE a verbatim block (not the title or label lines).
    /// This is used by the tokenizer to decide whether to emit VerbatimContent tokens
    /// or process the line as normal TXXT.
    ///
    /// Note: Title lines (e.g., "code:") and label lines (e.g., "(python)") return false
    /// because they should be processed as normal TXXT tokens.
    pub fn is_verbatim_content(&self, line_num: usize, blocks: &[VerbatimBlock]) -> bool {
        for block in blocks {
            // Check if line is strictly inside the block (not the start/end markers)
            // start_line = title line (process as TXXT)
            // end_line = label line (process as TXXT)
            // Everything in between = verbatim content (preserve exactly)
            if block.start_line < line_num && line_num < block.end_line {
                return true;
            }
        }
        false
    }
}

impl Default for VerbatimScanner {
    fn default() -> Self {
        Self::new()
    }
}
