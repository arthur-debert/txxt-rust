use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct VerbatimBlock {
    pub start_line: usize,   // 1-indexed line number where verbatim starts
    pub end_line: usize,     // 1-indexed line number where verbatim ends
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
                // Not in a verbatim block - check for start
                if self.verbatim_start_re.is_match(line) && !line.trim_end().ends_with("::") {
                    current_block = Some(VerbatimBlock {
                        start_line: line_num,
                        end_line: line_num, // Will be updated when we find the end
                        start_indent: indent,
                    });
                }
            } else {
                // In a verbatim block - check for end
                if let Some(ref mut block) = current_block {
                    if indent == block.start_indent && self.verbatim_end_re.is_match(line) {
                        // Found the end
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

    pub fn is_verbatim_content(&self, line_num: usize, blocks: &[VerbatimBlock]) -> bool {
        for block in blocks {
            // Check if line is strictly inside the block (not the start/end markers)
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
