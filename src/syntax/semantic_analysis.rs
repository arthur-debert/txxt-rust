//! Phase 1c: High-Level Token Analysis
//!
//! Converts scanner tokens into high-level tokens. This is the third step
//! of Phase 1 lexing, where we elevate the low-level scanner token stream
//! into a higher-level stream of semantic nodes.
//!
//! src/lexer/mod.rs has the full architecture overview.

use crate::cst::high_level_tokens::{
    HighLevelNumberingForm, HighLevelNumberingStyle, HighLevelToken, HighLevelTokenBuilder,
    HighLevelTokenList,
};
use crate::cst::primitives::ScannerTokenSequence;
use crate::cst::{Position, ScannerToken, SequenceMarkerType, SourceSpan};

/// High-level token analyzer for converting scanner tokens to high-level tokens
///
/// This analyzer takes a flat stream of scanner tokens and transforms them
/// into higher-level tokens that represent syntactic constructs.
pub struct SemanticAnalyzer;

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    /// Create a new high-level token analyzer instance
    pub fn new() -> Self {
        Self
    }

    /// Analyze scanner tokens and convert them to high-level tokens
    ///
    /// Takes a flat stream of scanner tokens and transforms them into
    /// high-level tokens that represent higher-level syntactic constructs.
    /// Structural tokens are passed through unchanged.
    ///
    /// # Arguments
    /// * `scanner_tokens` - The scanner token vector from Phase 1b
    ///
    /// # Returns
    /// * `Result<HighLevelTokenList, SemanticAnalysisError>` - The high-level token list
    pub fn analyze(
        &self,
        scanner_tokens: Vec<ScannerToken>,
    ) -> Result<HighLevelTokenList, SemanticAnalysisError> {
        let mut high_level_tokens = Vec::new();
        let mut i = 0;

        // THE WALL CONCEPT: Indentation Handling
        //
        // In txxt, indented content has a "wall" - the position where actual content starts.
        // This follows the same architecture as verbatim blocks (see docs/specs/elements/verbatim/verbatim.txxt).
        //
        // Example:
        //   1. Session Title
        //
        //       |This is content at the wall
        //       |Second line also at the wall
        //
        // The wall (marked by |) is at column 4 (relative to the session title).
        // The leading whitespace ("    ") is STRUCTURAL, not content.
        //
        // Scanner tokens preserve everything for position tracking:
        //   - Indent (structural marker: "we're now at indent level 1")
        //   - Whitespace("    ") (the physical spaces creating that indentation)
        //   - Text("This") (actual content starts here - at the wall)
        //
        // High-level tokens separate structure from content:
        //   - Indent (structural token)
        //   - PlainTextLine {
        //       indentation_chars: "    ",  // The wall padding (STRUCTURAL)
        //       content: "This is content..." // Content at the wall (SEMANTIC)
        //     }
        //
        // This ensures:
        // 1. Scanner tokens preserve exact positions for LSP features
        // 2. High-level tokens make structure explicit via indentation_chars
        // 3. Parser sees content at the wall, never sees structural padding
        //
        // CRITICAL: This must be consistent across ALL line-level elements:
        // - PlainTextLine (paragraphs)
        // - SequenceTextLine (lists, sessions)
        // - (Future: Annotations, Definitions when they become line-level)
        //
        // See tests/tokenizer/test_indentation_wall_consistency.rs for comprehensive validation.
        let mut pending_indentation = String::new(); // Captured leading whitespace "before the wall"

        while i < scanner_tokens.len() {
            let token = &scanner_tokens[i];

            match token {
                // Structural tokens - pass through unchanged and clear pending indentation
                ScannerToken::BlankLine { span, .. } => {
                    let tokens = ScannerTokenSequence::from_tokens(vec![token.clone()]);
                    high_level_tokens.push(HighLevelToken::BlankLine {
                        span: span.clone(),
                        tokens,
                    });
                    pending_indentation.clear(); // Reset after structural token
                    i += 1;
                }
                ScannerToken::Indent { span } => {
                    let tokens = ScannerTokenSequence::from_tokens(vec![token.clone()]);
                    high_level_tokens.push(HighLevelToken::Indent {
                        span: span.clone(),
                        tokens,
                    });
                    pending_indentation.clear(); // Reset after structural token
                    i += 1;
                }
                ScannerToken::Dedent { span } => {
                    let tokens = ScannerTokenSequence::from_tokens(vec![token.clone()]);
                    high_level_tokens.push(HighLevelToken::Dedent {
                        span: span.clone(),
                        tokens,
                    });
                    pending_indentation.clear(); // Reset after structural token
                    i += 1;
                }

                // Leading whitespace at line start - capture it as indentation padding
                // This applies after: Indent, BlankLine, Dedent, Newline, or start of file
                ScannerToken::Whitespace { content, .. }
                    if pending_indentation.is_empty()
                        && (i == 0
                            || matches!(
                                scanner_tokens.get(i - 1),
                                Some(ScannerToken::Indent { .. })
                                    | Some(ScannerToken::BlankLine { .. })
                                    | Some(ScannerToken::Dedent { .. })
                                    | Some(ScannerToken::Newline { .. })
                            )) =>
                {
                    // This whitespace is "before the wall" - capture it as indentation padding
                    pending_indentation = content.clone();
                    i += 1;
                    continue;
                }

                // Eof token - skip it (don't convert to empty TextSpan)
                ScannerToken::Eof { .. } => {
                    i += 1;
                    continue;
                }

                // Process line-level tokens
                _ => {
                    // First, try to recognize complex patterns (annotations, definitions, verbatim blocks)
                    if let Some((pattern_tokens, consumed)) =
                        self.recognize_complex_pattern(&scanner_tokens, i)?
                    {
                        let pattern_semantic_token =
                            self.transform_complex_pattern(pattern_tokens)?;
                        high_level_tokens.push(pattern_semantic_token);
                        i += consumed;
                    }
                    // Then check if this looks like a core block element (paragraph, session, list)
                    else if self.is_core_block_element(&scanner_tokens, i) {
                        let (line_tokens, consumed) =
                            self.extract_line_tokens(&scanner_tokens, i)?;
                        let line_semantic_token =
                            self.process_line_tokens(line_tokens, pending_indentation.clone())?;
                        high_level_tokens.push(line_semantic_token);
                        pending_indentation.clear(); // Used up the indentation
                        i += consumed;
                    } else {
                        // Process individual tokens for specific elements (annotations, definitions, etc.)
                        let token = &scanner_tokens[i];
                        match token {
                            // TxxtMarker transformation - Issue #81
                            ScannerToken::TxxtMarker { .. } => {
                                high_level_tokens.push(self.transform_txxt_marker(token)?);
                            }

                            // Label transformation - Issue #82
                            ScannerToken::Identifier { content, span } => {
                                high_level_tokens.push(self.transform_label(
                                    content.clone(),
                                    span.clone(),
                                    token,
                                )?);
                            }

                            // Text Span transformation - Issue #85
                            ScannerToken::Text { content, span } => {
                                high_level_tokens.push(self.transform_text_span(
                                    content.clone(),
                                    span.clone(),
                                    token,
                                )?);
                            }

                            // Sequence Marker transformation - Issue #84
                            ScannerToken::SequenceMarker { marker_type, span } => {
                                high_level_tokens.push(self.transform_sequence_marker(
                                    marker_type.clone(),
                                    span.clone(),
                                    token,
                                )?);
                            }

                            // Preserve syntactic markers instead of converting to text spans
                            ScannerToken::Colon { span } => {
                                // Preserve colon as a syntactic marker for parameter parsing
                                let tokens = ScannerTokenSequence::from_tokens(vec![token.clone()]);
                                high_level_tokens.push(HighLevelTokenBuilder::colon_with_tokens(
                                    span.clone(),
                                    tokens,
                                ));
                            }

                            // Handle other tokens as text spans for now
                            _ => {
                                // Convert other tokens to text spans as fallback
                                // This will be refined in subsequent transformation issues
                                let content = self.token_to_text_content(token);
                                high_level_tokens.push(
                                    HighLevelTokenBuilder::text_span_with_tokens(
                                        content,
                                        token.span().clone(),
                                        ScannerTokenSequence {
                                            tokens: vec![token.clone()],
                                        },
                                    ),
                                );
                            }
                        }
                        i += 1;
                    }
                }
            }
        }

        Ok(HighLevelTokenList::with_tokens(high_level_tokens))
    }

    /// Recognize complex patterns like definitions, annotations, and verbatim blocks
    ///
    /// This method looks ahead from the current position to identify patterns
    /// that should be transformed into complex semantic tokens instead of
    /// being processed as individual tokens.
    ///
    /// # Arguments
    /// * `scanner_tokens` - The full scanner token vector
    /// * `start_index` - The index to start checking from
    ///
    /// # Returns
    /// * `Result<Option<(Vec<ScannerToken>, usize)>, SemanticAnalysisError>` -
    ///   If a pattern is recognized, returns Some((pattern_tokens, tokens_consumed))
    ///   If no pattern is recognized, returns None
    fn recognize_complex_pattern(
        &self,
        scanner_tokens: &[ScannerToken],
        start_index: usize,
    ) -> Result<Option<(Vec<ScannerToken>, usize)>, SemanticAnalysisError> {
        if start_index >= scanner_tokens.len() {
            return Ok(None);
        }

        let token = &scanner_tokens[start_index];

        match token {
            // Check for definition pattern: Text + Whitespace + TxxtMarker
            ScannerToken::Text { .. } => {
                if let Some((tokens, consumed)) =
                    self.recognize_definition_pattern(scanner_tokens, start_index)?
                {
                    return Ok(Some((tokens, consumed)));
                }
            }

            // Check for annotation pattern: TxxtMarker + Whitespace + Identifier + ... + TxxtMarker
            ScannerToken::TxxtMarker { .. } => {
                if let Some((tokens, consumed)) =
                    self.recognize_annotation_pattern(scanner_tokens, start_index)?
                {
                    return Ok(Some((tokens, consumed)));
                }
            }

            // Check for verbatim block pattern: VerbatimTitle + IndentationWall + ... + VerbatimLabel
            ScannerToken::VerbatimTitle { .. } => {
                if let Some((tokens, consumed)) =
                    self.recognize_verbatim_block_pattern(scanner_tokens, start_index)?
                {
                    return Ok(Some((tokens, consumed)));
                }
            }

            // Check for verbatim block pattern starting with IndentationWall
            ScannerToken::IndentationWall { .. } => {
                if start_index + 1 < scanner_tokens.len()
                    && matches!(
                        scanner_tokens[start_index + 1],
                        ScannerToken::VerbatimTitle { .. }
                    )
                {
                    if let Some((tokens, consumed)) =
                        self.recognize_verbatim_block_pattern(scanner_tokens, start_index)?
                    {
                        return Ok(Some((tokens, consumed)));
                    }
                }
            }

            _ => {}
        }

        Ok(None)
    }

    /// Recognize definition pattern: Text + ... + TxxtMarker
    /// Supports both simple definitions (Text + Whitespace + TxxtMarker)
    /// and parameterized definitions (Text + Colon + Text + ... + Whitespace + TxxtMarker)
    fn recognize_definition_pattern(
        &self,
        scanner_tokens: &[ScannerToken],
        start_index: usize,
    ) -> Result<Option<(Vec<ScannerToken>, usize)>, SemanticAnalysisError> {
        if start_index + 2 >= scanner_tokens.len() {
            return Ok(None);
        }

        // Must start with Text
        if !matches!(scanner_tokens[start_index], ScannerToken::Text { .. }) {
            return Ok(None);
        }

        // Look for the closing TxxtMarker, but be more restrictive about what we accept
        let mut consumed = 1;
        let mut i = start_index + 1;
        let mut has_whitespace = false;

        while i < scanner_tokens.len() {
            let token = &scanner_tokens[i];
            match token {
                ScannerToken::TxxtMarker { .. } => {
                    // Found the closing TxxtMarker - this is a definition
                    consumed += 1;
                    break;
                }
                ScannerToken::Whitespace { .. } => {
                    has_whitespace = true;
                    consumed += 1;
                    i += 1;
                }
                ScannerToken::Colon { .. } => {
                    // Allow colon for parameterized definitions
                    consumed += 1;
                    i += 1;
                }
                ScannerToken::Text { .. } => {
                    // Allow additional text for parameterized definitions
                    consumed += 1;
                    i += 1;
                }
                ScannerToken::Newline { .. } => {
                    // Stop at line boundaries
                    break;
                }
                ScannerToken::BlankLine { .. }
                | ScannerToken::Indent { .. }
                | ScannerToken::Dedent { .. } => {
                    // Stop at structural tokens
                    break;
                }
                _ => {
                    // Don't include other token types (like Identifier) in definitions
                    break;
                }
            }
        }

        // Must have found a TxxtMarker and have proper structure
        if consumed >= 3
            && matches!(
                scanner_tokens[start_index + consumed - 1],
                ScannerToken::TxxtMarker { .. }
            )
            && (has_whitespace || consumed > 3)
        // Either has whitespace or has parameters
        {
            // Check if there's a trailing newline after the TxxtMarker and consume it
            // This prevents it from being converted to a TextSpan in the main loop
            let mut final_consumed = consumed;
            if start_index + consumed < scanner_tokens.len()
                && matches!(
                    scanner_tokens[start_index + consumed],
                    ScannerToken::Newline { .. }
                )
            {
                final_consumed += 1; // Consume the newline
            }

            let pattern_tokens = scanner_tokens[start_index..start_index + consumed].to_vec();
            return Ok(Some((pattern_tokens, final_consumed)));
        }

        Ok(None)
    }

    /// Recognize annotation pattern: TxxtMarker + Whitespace + Identifier + ... + TxxtMarker
    fn recognize_annotation_pattern(
        &self,
        scanner_tokens: &[ScannerToken],
        start_index: usize,
    ) -> Result<Option<(Vec<ScannerToken>, usize)>, SemanticAnalysisError> {
        if start_index + 4 >= scanner_tokens.len() {
            return Ok(None);
        }

        // Look for opening TxxtMarker
        if !matches!(scanner_tokens[start_index], ScannerToken::TxxtMarker { .. }) {
            return Ok(None);
        }

        // Look for pattern: TxxtMarker + Whitespace + (Identifier|Text) + Whitespace + TxxtMarker
        if matches!(
            scanner_tokens[start_index + 1],
            ScannerToken::Whitespace { .. }
        ) && matches!(
            scanner_tokens[start_index + 2],
            ScannerToken::Identifier { .. } | ScannerToken::Text { .. }
        ) && matches!(
            scanner_tokens[start_index + 3],
            ScannerToken::Whitespace { .. }
        ) && matches!(
            scanner_tokens[start_index + 4],
            ScannerToken::TxxtMarker { .. }
        ) {
            // Extract tokens up to the closing TxxtMarker (minimum 5 tokens)
            let mut consumed = 5;

            // Look for optional content after the closing TxxtMarker
            let mut i = start_index + 5;
            while i < scanner_tokens.len() {
                let token = &scanner_tokens[i];
                match token {
                    ScannerToken::Newline { .. } => {
                        consumed += 1;
                        break;
                    }
                    ScannerToken::BlankLine { .. }
                    | ScannerToken::Indent { .. }
                    | ScannerToken::Dedent { .. } => {
                        // Stop at structural tokens
                        break;
                    }
                    _ => {
                        consumed += 1;
                        i += 1;
                    }
                }
            }

            let pattern_tokens = scanner_tokens[start_index..start_index + consumed].to_vec();
            return Ok(Some((pattern_tokens, consumed)));
        }

        Ok(None)
    }

    /// Recognize verbatim block pattern: VerbatimTitle + IndentationWall + ... + VerbatimLabel
    fn recognize_verbatim_block_pattern(
        &self,
        scanner_tokens: &[ScannerToken],
        start_index: usize,
    ) -> Result<Option<(Vec<ScannerToken>, usize)>, SemanticAnalysisError> {
        if start_index + 2 >= scanner_tokens.len() {
            return Ok(None);
        }

        // Check for either:
        // 1. VerbatimTitle + IndentationWall
        // 2. IndentationWall + VerbatimTitle
        let has_verbatim_pattern = (matches!(
            scanner_tokens[start_index],
            ScannerToken::VerbatimTitle { .. }
        ) && matches!(
            scanner_tokens[start_index + 1],
            ScannerToken::IndentationWall { .. }
        )) || (matches!(
            scanner_tokens[start_index],
            ScannerToken::IndentationWall { .. }
        ) && matches!(
            scanner_tokens[start_index + 1],
            ScannerToken::VerbatimTitle { .. }
        ));

        if has_verbatim_pattern {
            // Look for VerbatimLabel (may have content in between)
            let mut consumed = 2;
            let mut i = start_index + 2;

            while i < scanner_tokens.len() {
                let token = &scanner_tokens[i];
                match token {
                    ScannerToken::VerbatimLabel { .. } => {
                        consumed += 1;
                        break;
                    }
                    ScannerToken::IgnoreTextSpan { .. } => {
                        consumed += 1;
                        i += 1;
                    }
                    _ => {
                        // Stop if we encounter unexpected tokens
                        break;
                    }
                }
            }

            if consumed >= 3 {
                // Must have at least (VerbatimTitle + IndentationWall OR IndentationWall + VerbatimTitle) + VerbatimLabel
                let pattern_tokens = scanner_tokens[start_index..start_index + consumed].to_vec();
                return Ok(Some((pattern_tokens, consumed)));
            }
        }

        Ok(None)
    }

    /// Transform complex pattern tokens into semantic tokens
    ///
    /// This method determines which transformation to apply based on the
    /// pattern of tokens and calls the appropriate transformation function.
    ///
    /// # Arguments
    /// * `pattern_tokens` - The tokens that form a complex pattern
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The transformed semantic token
    fn transform_complex_pattern(
        &self,
        pattern_tokens: Vec<ScannerToken>,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        if pattern_tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Cannot transform empty pattern".to_string(),
            ));
        }

        let first_token = &pattern_tokens[0];

        match first_token {
            // Definition pattern: Text + ... + TxxtMarker
            ScannerToken::Text { .. } => {
                if pattern_tokens.len() >= 3
                    && matches!(
                        pattern_tokens[pattern_tokens.len() - 1],
                        ScannerToken::TxxtMarker { .. }
                    )
                {
                    let span = SourceSpan {
                        start: pattern_tokens[0].span().start,
                        end: pattern_tokens[pattern_tokens.len() - 1].span().end,
                    };
                    return self.transform_definition(pattern_tokens, span);
                }
            }

            // Annotation pattern: TxxtMarker + ... + TxxtMarker
            ScannerToken::TxxtMarker { .. } => {
                if pattern_tokens.len() >= 5
                    && matches!(pattern_tokens[1], ScannerToken::Whitespace { .. })
                    && matches!(
                        pattern_tokens[2],
                        ScannerToken::Identifier { .. } | ScannerToken::Text { .. }
                    )
                    && matches!(pattern_tokens[3], ScannerToken::Whitespace { .. })
                    && matches!(pattern_tokens[4], ScannerToken::TxxtMarker { .. })
                {
                    let span = SourceSpan {
                        start: pattern_tokens[0].span().start,
                        end: pattern_tokens[pattern_tokens.len() - 1].span().end,
                    };
                    return self.transform_annotation(pattern_tokens, span);
                }
            }

            // Verbatim block pattern: VerbatimTitle + IndentationWall + ... + VerbatimLabel
            ScannerToken::VerbatimTitle { .. } => {
                if pattern_tokens.len() >= 3
                    && matches!(pattern_tokens[1], ScannerToken::IndentationWall { .. })
                    && matches!(
                        pattern_tokens[pattern_tokens.len() - 1],
                        ScannerToken::VerbatimLabel { .. }
                    )
                {
                    let span = SourceSpan {
                        start: pattern_tokens[0].span().start,
                        end: pattern_tokens[pattern_tokens.len() - 1].span().end,
                    };
                    return self.transform_verbatim_block(pattern_tokens, span);
                }
            }

            // Verbatim block pattern: IndentationWall + VerbatimTitle + ... + VerbatimLabel
            ScannerToken::IndentationWall { .. } => {
                if pattern_tokens.len() >= 3
                    && matches!(pattern_tokens[1], ScannerToken::VerbatimTitle { .. })
                    && matches!(
                        pattern_tokens[pattern_tokens.len() - 1],
                        ScannerToken::VerbatimLabel { .. }
                    )
                {
                    let span = SourceSpan {
                        start: pattern_tokens[0].span().start,
                        end: pattern_tokens[pattern_tokens.len() - 1].span().end,
                    };
                    return self.transform_verbatim_block(pattern_tokens, span);
                }
            }

            _ => {}
        }

        Err(SemanticAnalysisError::AnalysisError(
            "Unable to determine pattern type for transformation".to_string(),
        ))
    }

    /// Check if the current position looks like a core block element (paragraph, session, list)
    ///
    /// This method detects when we should apply line-level grouping vs individual
    /// token transformation. Core block elements are paragraphs, sessions, and lists
    /// that should be processed as complete lines.
    ///
    /// Uses proper local pattern matching instead of flawed global heuristics.
    ///
    /// # Arguments
    /// * `scanner_tokens` - The full scanner token vector
    /// * `start_index` - The index to start checking from
    ///
    /// # Returns
    /// * `bool` - True if this looks like a core block element
    fn is_core_block_element(&self, scanner_tokens: &[ScannerToken], start_index: usize) -> bool {
        if start_index >= scanner_tokens.len() {
            return false;
        }

        let token = &scanner_tokens[start_index];

        match token {
            // SequenceMarker tokens should be processed as line-level elements for lists/sessions
            ScannerToken::SequenceMarker { .. } => {
                // Look ahead to see if this line has content after the marker
                let (line_tokens, _) = match self.extract_line_tokens(scanner_tokens, start_index) {
                    Ok(result) => result,
                    Err(_) => return false,
                };

                // If the line has content after the sequence marker, it's a list/session item
                line_tokens.len() > 1
            }

            // Text tokens should be processed as line-level elements for paragraphs
            ScannerToken::Text { .. } => {
                // Look ahead to see if this line contains any special markers
                let (line_tokens, _) = match self.extract_line_tokens(scanner_tokens, start_index) {
                    Ok(result) => result,
                    Err(_) => return false,
                };

                // If the line contains TxxtMarkers, it's not a simple paragraph - use individual processing
                let has_txxt_markers = line_tokens
                    .iter()
                    .any(|token| matches!(token, ScannerToken::TxxtMarker { .. }));

                if has_txxt_markers {
                    return false;
                }

                // If the line contains sequence markers, it's not a simple paragraph
                let has_sequence_markers = line_tokens
                    .iter()
                    .any(|token| matches!(token, ScannerToken::SequenceMarker { .. }));

                if has_sequence_markers {
                    return false;
                }

                // Otherwise, this looks like a paragraph - use line-level processing
                // Note: Colons are allowed in paragraphs (e.g., "features:" in regular text).
                // Definitions are detected by TxxtMarkers (::), which are already filtered above.
                true
            }

            // All other tokens should use individual processing
            _ => false,
        }
    }

    /// Extract tokens that belong to a single line
    ///
    /// This method groups scanner tokens by line, stopping at line boundaries
    /// (Newline tokens or end of input).
    ///
    /// # Arguments
    /// * `scanner_tokens` - The full scanner token vector
    /// * `start_index` - The index to start extracting from
    ///
    /// # Returns
    /// * `Result<(Vec<ScannerToken>, usize), SemanticAnalysisError>` - Tuple of (line_tokens, tokens_consumed)
    fn extract_line_tokens(
        &self,
        scanner_tokens: &[ScannerToken],
        start_index: usize,
    ) -> Result<(Vec<ScannerToken>, usize), SemanticAnalysisError> {
        let mut line_tokens = Vec::new();
        let mut i = start_index;

        while i < scanner_tokens.len() {
            let token = &scanner_tokens[i];

            match token {
                // Stop at line boundaries
                ScannerToken::Newline { .. } => {
                    line_tokens.push(token.clone());
                    i += 1;
                    break;
                }
                // Stop at structural tokens (they're handled separately)
                ScannerToken::BlankLine { .. }
                | ScannerToken::Indent { .. }
                | ScannerToken::Dedent { .. } => {
                    break;
                }
                // Include all other tokens in the line
                _ => {
                    line_tokens.push(token.clone());
                    i += 1;
                }
            }
        }

        // If we reached the end without finding a newline, that's the last line
        let consumed = i - start_index;
        Ok((line_tokens, consumed))
    }

    /// Process a line of scanner tokens into a semantic token
    ///
    /// This method analyzes a line of tokens and creates the appropriate
    /// line-level semantic token (PlainTextLine or SequenceTextLine).
    ///
    /// # Arguments
    /// * `line_tokens` - The scanner tokens for a single line
    /// * `indentation_chars` - Leading whitespace before this line (from pending_indentation)
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The line-level semantic token
    fn process_line_tokens(
        &self,
        line_tokens: Vec<ScannerToken>,
        indentation_chars: String,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        if line_tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Cannot process empty line tokens".to_string(),
            ));
        }

        // Check if this line starts with a sequence marker
        if let Some(first_token) = line_tokens.first() {
            if matches!(first_token, ScannerToken::SequenceMarker { .. }) {
                return self.create_sequence_text_line(line_tokens, indentation_chars);
            }
        }

        // Otherwise, create a plain text line
        self.create_plain_text_line(line_tokens, indentation_chars)
    }

    /// Create a SequenceTextLine semantic token from line tokens
    fn create_sequence_text_line(
        &self,
        line_tokens: Vec<ScannerToken>,
        indentation_chars: String,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        if line_tokens.len() < 2 {
            return Err(SemanticAnalysisError::AnalysisError(
                "SequenceTextLine requires at least a marker and some content".to_string(),
            ));
        }

        let marker_token = line_tokens[0].clone();
        let content_tokens = line_tokens[1..].to_vec();

        // Transform the sequence marker
        let marker_semantic = match &marker_token {
            ScannerToken::SequenceMarker { marker_type, span } => {
                self.transform_sequence_marker(marker_type.clone(), span.clone(), &marker_token)?
            }
            _ => {
                return Err(SemanticAnalysisError::AnalysisError(
                    "Expected SequenceMarker as first token".to_string(),
                ));
            }
        };

        // Transform the content tokens into a single text span
        let content_semantic = self.tokens_to_text_span_line_level(content_tokens)?;

        // Calculate the span for the entire line
        let start_span = marker_token.span();
        let end_span = line_tokens.last().unwrap().span();
        let line_span = SourceSpan {
            start: start_span.start,
            end: end_span.end,
        };

        Ok(HighLevelTokenBuilder::sequence_text_line(
            indentation_chars,
            marker_semantic,
            content_semantic,
            line_span,
        ))
    }

    /// Create a PlainTextLine semantic token from line tokens
    fn create_plain_text_line(
        &self,
        line_tokens: Vec<ScannerToken>,
        indentation_chars: String,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Transform all tokens into a single text span
        let content_semantic = self.tokens_to_text_span_line_level(line_tokens.clone())?;

        // Calculate the span for the entire line
        let start_span = line_tokens.first().unwrap().span();
        let end_span = line_tokens.last().unwrap().span();
        let line_span = SourceSpan {
            start: start_span.start,
            end: end_span.end,
        };

        Ok(HighLevelTokenBuilder::plain_text_line(
            indentation_chars,
            content_semantic,
            line_span,
        ))
    }

    /// Convert a list of scanner tokens into a single TextSpan semantic token (for line-level processing)
    fn tokens_to_text_span_line_level(
        &self,
        tokens: Vec<ScannerToken>,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        let mut content = String::new();
        let mut start_span = None;
        let mut end_span = None;
        let token_sequence = tokens.clone();

        for token in tokens {
            if start_span.is_none() {
                start_span = Some(token.span().start);
            }
            end_span = Some(token.span().end);

            // Convert token to text content, preserving whitespace for line-level processing
            let token_content = self.token_to_text_content(&token);
            content.push_str(&token_content);
        }

        let span = SourceSpan {
            start: start_span.unwrap_or(Position { row: 0, column: 0 }),
            end: end_span.unwrap_or(Position { row: 0, column: 0 }),
        };

        Ok(HighLevelTokenBuilder::text_span_with_tokens(
            content,
            span,
            ScannerTokenSequence {
                tokens: token_sequence,
            },
        ))
    }

    /// Convert a list of scanner tokens into a single TextSpan semantic token (for individual processing)
    fn tokens_to_text_span(
        &self,
        tokens: Vec<ScannerToken>,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        let mut content = String::new();
        let mut start_span = None;
        let mut end_span = None;
        let token_sequence = tokens.clone();

        for token in tokens {
            if start_span.is_none() {
                start_span = Some(token.span().start);
            }
            end_span = Some(token.span().end);

            // Convert token to text content, but filter out whitespace tokens
            match &token {
                ScannerToken::Whitespace { .. } => {
                    // Skip whitespace tokens when combining
                    continue;
                }
                _ => {
                    let token_content = self.token_to_text_content(&token);
                    content.push_str(&token_content);
                }
            }
        }

        let span = SourceSpan {
            start: start_span.unwrap_or(Position { row: 0, column: 0 }),
            end: end_span.unwrap_or(Position { row: 0, column: 0 }),
        };

        Ok(HighLevelTokenBuilder::text_span_with_tokens(
            content,
            span,
            ScannerTokenSequence {
                tokens: token_sequence,
            },
        ))
    }

    /// Transform TxxtMarker scanner token to semantic token
    ///
    /// This implements the TxxtMarker transformation as specified in Issue #81.
    /// TxxtMarker tokens represent the fundamental :: markers used across
    /// annotations, definitions, and verbatim blocks.
    ///
    /// # Arguments
    /// * `token` - The TxxtMarker scanner token
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_txxt_marker(
        &self,
        token: &ScannerToken,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        match token {
            ScannerToken::TxxtMarker { span } => {
                // Transform TxxtMarker scanner token to TxxtMarker semantic token
                // This preserves the fundamental :: marker information for use
                // in subsequent parsing phases

                // Preserve the source token
                let tokens = ScannerTokenSequence::from_tokens(vec![token.clone()]);

                Ok(HighLevelTokenBuilder::txxt_marker_with_tokens(
                    span.clone(),
                    tokens,
                ))
            }
            _ => Err(SemanticAnalysisError::InvalidTokenType {
                expected: "TxxtMarker".to_string(),
                actual: format!("{:?}", token),
            }),
        }
    }

    /// Transform Identifier scanner token to Label semantic token
    ///
    /// This implements the Label transformation as specified in Issue #82.
    /// Label tokens represent structured identifiers used in annotations and
    /// verbatim blocks, supporting namespaced identifiers like "python",
    /// "org.example.custom".
    ///
    /// # Arguments
    /// * `content` - The identifier content from the scanner token
    /// * `span` - The source span of the identifier
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_label(
        &self,
        content: String,
        span: SourceSpan,
        source_token: &ScannerToken,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Validate that the content is a valid label
        if content.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Label content cannot be empty".to_string(),
            ));
        }

        // Check if the label starts with a valid character
        if let Some(first_char) = content.chars().next() {
            if !self.is_valid_label_start(first_char) {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Label must start with a letter, got '{}'",
                    first_char
                )));
            }
        }

        // Validate all characters in the label
        for (i, c) in content.chars().enumerate() {
            if !self.is_valid_label_char(c) && c != '.' {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Invalid character '{}' at position {} in label '{}'",
                    c, i, content
                )));
            }
        }

        // Preserve source token
        let tokens = ScannerTokenSequence::from_tokens(vec![source_token.clone()]);

        // Transform Identifier scanner token to Label semantic token
        Ok(HighLevelTokenBuilder::label_with_tokens(
            content, span, tokens,
        ))
    }

    /// Check if a character is valid at the start of a label
    pub fn is_valid_label_start(&self, c: char) -> bool {
        c.is_ascii_alphabetic()
    }

    /// Check if a character is valid within a label
    pub fn is_valid_label_char(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }

    /// Transform Text scanner token to TextSpan semantic token
    ///
    /// This implements the Text Span transformation as specified in Issue #85.
    /// TextSpan tokens represent basic text content without special formatting,
    /// serving as building blocks for larger line constructs.
    ///
    /// # Arguments
    /// * `content` - The text content from the scanner token
    /// * `span` - The source span of the text
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_text_span(
        &self,
        content: String,
        span: SourceSpan,
        source_token: &ScannerToken,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Validate that the content is not empty
        if content.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Text span content cannot be empty".to_string(),
            ));
        }

        // Preserve source token
        let tokens = ScannerTokenSequence::from_tokens(vec![source_token.clone()]);

        // Transform Text scanner token to TextSpan semantic token
        // This preserves the basic text content for use in subsequent parsing phases
        Ok(HighLevelTokenBuilder::text_span_with_tokens(
            content, span, tokens,
        ))
    }

    /// Transform SequenceMarker scanner token to SequenceMarker semantic token
    ///
    /// This implements the Sequence Marker transformation as specified in Issue #84.
    /// SequenceMarker tokens represent list and session numbering components,
    /// handling numeric (1.), alphabetic (a.), roman (i.), and plain (-) markers,
    /// in both regular (2.) and extended (1.3.b) forms.
    ///
    /// # Arguments
    /// * `marker_type` - The sequence marker type from the scanner token
    /// * `span` - The source span of the marker
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_sequence_marker(
        &self,
        marker_type: SequenceMarkerType,
        span: SourceSpan,
        source_token: &ScannerToken,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        let (style, form) = self.classify_sequence_marker(&marker_type);
        let marker_text = marker_type.content().to_string();

        // Preserve source token
        let tokens = ScannerTokenSequence::from_tokens(vec![source_token.clone()]);

        // Transform SequenceMarker scanner token to SequenceMarker semantic token
        Ok(HighLevelTokenBuilder::sequence_marker_with_tokens(
            style,
            form,
            marker_text,
            span,
            tokens,
        ))
    }

    /// Classify a sequence marker type into semantic numbering style and form
    ///
    /// This helper method determines the semantic numbering style and form
    /// based on the scanner token's sequence marker type.
    ///
    /// # Arguments
    /// * `marker_type` - The sequence marker type to classify
    ///
    /// # Returns
    /// * `(HighLevelNumberingStyle, HighLevelNumberingForm)` - The classified style and form
    pub fn classify_sequence_marker(
        &self,
        marker_type: &SequenceMarkerType,
    ) -> (HighLevelNumberingStyle, HighLevelNumberingForm) {
        match marker_type {
            SequenceMarkerType::Plain(_) => (
                HighLevelNumberingStyle::Plain,
                HighLevelNumberingForm::Regular,
            ),
            SequenceMarkerType::Numerical(_, _) => (
                HighLevelNumberingStyle::Numeric,
                HighLevelNumberingForm::Regular,
            ),
            SequenceMarkerType::Alphabetical(_, _) => (
                HighLevelNumberingStyle::Alphabetic,
                HighLevelNumberingForm::Regular,
            ),
            SequenceMarkerType::Roman(_, _) => (
                HighLevelNumberingStyle::Roman,
                HighLevelNumberingForm::Regular,
            ),
        }
    }

    /// Transform a sequence of text tokens into a PlainTextLine semantic token
    /// PlainTextLine tokens represent simple text content without special markers
    /// or structure, containing a single TextSpan component.
    ///
    /// # Arguments
    /// * `text_tokens` - Vector of Text scanner tokens that form a line
    /// * `line_span` - The source span covering the entire line
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_plain_text_line(
        &self,
        text_tokens: Vec<ScannerToken>,
        line_span: SourceSpan,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Validate that we have at least one text token
        if text_tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Plain text line must contain at least one text token".to_string(),
            ));
        }

        // Validate that all tokens are Text tokens
        for token in &text_tokens {
            if !matches!(token, ScannerToken::Text { .. }) {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Plain text line can only contain Text tokens, got {:?}",
                    token
                )));
            }
        }

        // Combine all text content into a single string
        let combined_content = text_tokens
            .iter()
            .map(|token| {
                if let ScannerToken::Text { content, .. } = token {
                    content.as_str()
                } else {
                    "" // This should never happen due to validation above
                }
            })
            .collect::<Vec<&str>>()
            .join("");

        // Aggregate all source tokens for preservation
        let aggregated_tokens = ScannerTokenSequence::from_tokens(text_tokens.clone());

        // Create a single TextSpan for the combined content with aggregated tokens
        let text_span = HighLevelTokenBuilder::text_span_with_tokens(
            combined_content,
            line_span.clone(),
            aggregated_tokens.clone(),
        );

        // Transform to PlainTextLine semantic token with aggregated tokens
        // Note: This is a legacy method, indentation should be handled by the caller
        Ok(HighLevelTokenBuilder::plain_text_line_with_tokens(
            String::new(),
            text_span,
            line_span,
            aggregated_tokens,
        ))
    }

    /// Transform a sequence marker followed by text content into a SequenceTextLine semantic token
    ///
    /// This implements the Sequence Text Line transformation as specified in Issue #86.
    /// SequenceTextLine tokens represent lines beginning with sequence markers followed
    /// by text content, combining Sequence Marker and Text Span components.
    ///
    /// # Arguments
    /// * `marker_token` - The sequence marker semantic token
    /// * `text_tokens` - Vector of Text scanner tokens that form the content
    /// * `line_span` - The source span covering the entire line
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_sequence_text_line(
        &self,
        marker_token: HighLevelToken,
        text_tokens: Vec<ScannerToken>,
        line_span: SourceSpan,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Validate that we have at least one text token
        if text_tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Sequence text line must contain at least one text token".to_string(),
            ));
        }

        // Validate that all tokens are Text tokens
        for token in &text_tokens {
            if !matches!(token, ScannerToken::Text { .. }) {
                return Err(SemanticAnalysisError::AnalysisError(format!(
                    "Sequence text line can only contain Text tokens, got {:?}",
                    token
                )));
            }
        }

        // Validate that the marker token is a SequenceMarker
        if !matches!(marker_token, HighLevelToken::SequenceMarker { .. }) {
            return Err(SemanticAnalysisError::AnalysisError(format!(
                "Sequence text line marker must be a SequenceMarker token, got {:?}",
                marker_token
            )));
        }

        // Combine all text content into a single string
        let combined_content = text_tokens
            .iter()
            .map(|token| {
                if let ScannerToken::Text { content, .. } = token {
                    content.as_str()
                } else {
                    "" // This should never happen due to validation above
                }
            })
            .collect::<Vec<&str>>()
            .join("");

        // Aggregate source tokens: marker tokens + text tokens
        let mut all_tokens = marker_token.tokens().tokens.clone();
        all_tokens.extend(text_tokens.clone());
        let aggregated_tokens = ScannerTokenSequence::from_tokens(all_tokens);

        // Create a single TextSpan for the combined content with text tokens
        let text_tokens_seq = ScannerTokenSequence::from_tokens(text_tokens.clone());
        let text_span = HighLevelTokenBuilder::text_span_with_tokens(
            combined_content,
            line_span.clone(),
            text_tokens_seq,
        );

        // Transform to SequenceTextLine semantic token with all aggregated tokens
        // Note: This is a legacy method, indentation should be handled by the caller
        Ok(HighLevelTokenBuilder::sequence_text_line_with_tokens(
            String::new(),
            marker_token,
            text_span,
            line_span,
            aggregated_tokens,
        ))
    }

    /// Transform scanner tokens into an Annotation semantic token
    ///
    /// This implements the Annotation transformation as specified in Issue #88.
    /// Annotation tokens represent metadata elements that attach structured information
    /// to other elements. They follow the pattern:
    /// TxxtMarker + Whitespace + Text (label) + Whitespace + TxxtMarker + Text? (content)
    ///
    /// # Arguments
    /// * `tokens` - Vector of scanner tokens that form an annotation
    /// * `span` - The source span covering the entire annotation
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_annotation(
        &self,
        tokens: Vec<ScannerToken>,
        span: SourceSpan,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Validate minimum annotation structure: :: label ::
        if tokens.len() < 5 {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation must have at least 5 tokens: :: label ::".to_string(),
            ));
        }

        // Check for opening TxxtMarker
        if !matches!(tokens[0], ScannerToken::TxxtMarker { .. }) {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation must start with TxxtMarker".to_string(),
            ));
        }

        // Check for closing TxxtMarker (should be at position 4 for basic annotation)
        let closing_marker_pos = self.find_closing_txxt_marker(&tokens)?;
        if closing_marker_pos < 4 {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation must have proper structure: :: label ::".to_string(),
            ));
        }

        // Extract label (between first TxxtMarker and closing TxxtMarker)
        let label_tokens = &tokens[2..closing_marker_pos];
        let (label_token, parameters) = self.parse_label_with_parameters(label_tokens)?;

        // Extract content (after closing TxxtMarker, if any)
        let content = if closing_marker_pos + 1 < tokens.len() {
            let content_tokens = &tokens[closing_marker_pos + 1..];
            Some(self.parse_annotation_content(content_tokens)?)
        } else {
            None
        };

        // Aggregate all source tokens for preservation
        let aggregated_tokens = ScannerTokenSequence::from_tokens(tokens);

        // Transform to Annotation semantic token with aggregated tokens
        Ok(HighLevelTokenBuilder::annotation_with_tokens(
            label_token,
            parameters,
            content,
            span,
            aggregated_tokens,
        ))
    }

    /// Transform scanner tokens into a Definition semantic token
    ///
    /// This implements the Definition transformation as specified in Issue #88.
    /// Definition tokens represent structured elements that define terms, concepts,
    /// and entities. They follow the pattern:
    /// Text + Whitespace + TxxtMarker
    ///
    /// # Arguments
    /// * `tokens` - Vector of scanner tokens that form a definition
    /// * `span` - The source span covering the entire definition
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_definition(
        &self,
        tokens: Vec<ScannerToken>,
        span: SourceSpan,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Validate minimum definition structure: term ::
        if tokens.len() < 3 {
            return Err(SemanticAnalysisError::AnalysisError(
                "Definition must have at least 3 tokens: term ::".to_string(),
            ));
        }

        // Check for closing TxxtMarker (should be at the end)
        let txxt_marker_pos = tokens.len() - 1;
        if !matches!(tokens[txxt_marker_pos], ScannerToken::TxxtMarker { .. }) {
            return Err(SemanticAnalysisError::AnalysisError(
                "Definition must end with TxxtMarker".to_string(),
            ));
        }

        // Extract term (everything before the final TxxtMarker)
        let term_tokens = &tokens[..txxt_marker_pos];
        let (term_token, parameters) = self.parse_definition_term_with_parameters(term_tokens)?;

        // Aggregate all source tokens for preservation
        let aggregated_tokens = ScannerTokenSequence::from_tokens(tokens);

        // Transform to Definition semantic token with aggregated tokens
        Ok(HighLevelTokenBuilder::definition_with_tokens(
            term_token,
            parameters,
            span,
            aggregated_tokens,
        ))
    }

    /// Transform scanner tokens into a VerbatimBlock semantic token
    ///
    /// This implements the VerbatimBlock transformation as specified in Issue #89.
    /// VerbatimBlock tokens represent content that preserves exact formatting and spacing
    /// using the wall architecture pattern:
    /// VerbatimTitle + IndentationWall + IgnoreTextSpan + VerbatimLabel
    ///
    /// # Arguments
    /// * `tokens` - Vector of scanner tokens that form a verbatim block
    /// * `span` - The source span covering the entire verbatim block
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    pub fn transform_verbatim_block(
        &self,
        tokens: Vec<ScannerToken>,
        span: SourceSpan,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Validate minimum verbatim block structure: VerbatimTitle + IndentationWall + VerbatimLabel (empty blocks allowed)
        if tokens.len() < 3 {
            return Err(SemanticAnalysisError::AnalysisError(
                "VerbatimBlock must have at least 3 tokens: VerbatimTitle + IndentationWall + VerbatimLabel".to_string(),
            ));
        }

        // Extract components - handle both orderings:
        // 1. VerbatimTitle + IndentationWall
        // 2. IndentationWall + VerbatimTitle
        let mut i = 0;
        let title_token;
        let wall_token;
        let wall_type;

        if let ScannerToken::VerbatimTitle { .. } = &tokens[i] {
            // Order 1: VerbatimTitle first
            title_token = HighLevelTokenBuilder::text_span_with_tokens(
                tokens[i].content().to_string(),
                tokens[i].span().clone(),
                ScannerTokenSequence {
                    tokens: vec![tokens[i].clone()],
                },
            );
            i += 1;

            if let ScannerToken::IndentationWall { wall_type: wt, .. } = &tokens[i] {
                wall_token = HighLevelTokenBuilder::text_span_with_tokens(
                    "".to_string(), // Wall is structural, no content
                    tokens[i].span().clone(),
                    ScannerTokenSequence {
                        tokens: vec![tokens[i].clone()],
                    },
                );
                wall_type = wt.clone();
                i += 1;
            } else {
                return Err(SemanticAnalysisError::AnalysisError(
                    "VerbatimBlock must have IndentationWall after VerbatimTitle".to_string(),
                ));
            }
        } else if let ScannerToken::IndentationWall { wall_type: wt, .. } = &tokens[i] {
            // Order 2: IndentationWall first
            wall_token = HighLevelTokenBuilder::text_span_with_tokens(
                "".to_string(), // Wall is structural, no content
                tokens[i].span().clone(),
                ScannerTokenSequence {
                    tokens: vec![tokens[i].clone()],
                },
            );
            wall_type = wt.clone();
            i += 1;

            if let ScannerToken::VerbatimTitle { .. } = &tokens[i] {
                title_token = HighLevelTokenBuilder::text_span_with_tokens(
                    tokens[i].content().to_string(),
                    tokens[i].span().clone(),
                    ScannerTokenSequence {
                        tokens: vec![tokens[i].clone()],
                    },
                );
                i += 1;
            } else {
                return Err(SemanticAnalysisError::AnalysisError(
                    "VerbatimBlock must have VerbatimTitle after IndentationWall".to_string(),
                ));
            }
        } else {
            return Err(SemanticAnalysisError::AnalysisError(
                "VerbatimBlock must start with VerbatimTitle or IndentationWall".to_string(),
            ));
        }

        // 3. IgnoreTextSpan (may be multiple tokens, or empty for empty verbatim blocks)
        let mut content_tokens = Vec::new();
        while i < tokens.len() && matches!(tokens[i], ScannerToken::IgnoreTextSpan { .. }) {
            content_tokens.push(tokens[i].clone());
            i += 1;
        }

        // Create content token (empty verbatim blocks are allowed per specification)
        let content_token = if content_tokens.is_empty() {
            // Create empty content token for empty verbatim blocks
            let span = SourceSpan {
                start: tokens[i - 1].span().end, // Start after the wall
                end: tokens[i - 1].span().end,   // Same position (empty)
            };
            HighLevelTokenBuilder::text_span_with_tokens(
                "".to_string(),
                span.clone(),
                ScannerTokenSequence { tokens: vec![] }, // Empty tokens for empty content
            )
        } else {
            // Combine all content tokens into a single TextSpan (preserve exact whitespace for verbatim)
            self.tokens_to_text_span_exact(&content_tokens)?
        };

        // 4. VerbatimLabel (should be the last token)
        let (label_token, parameters) = if i < tokens.len() {
            if let ScannerToken::VerbatimLabel { .. } = &tokens[i] {
                // For verbatim labels, check if they contain parameters (colon separator)
                let label_content = tokens[i].content();
                if label_content.contains(':') {
                    // Split by colon to separate label from parameters
                    let parts: Vec<&str> = label_content.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let label_text = parts[0];
                        let param_text = parts[1];

                        let label_semantic_token = HighLevelTokenBuilder::text_span_with_tokens(
                            label_text.to_string(),
                            tokens[i].span().clone(),
                            ScannerTokenSequence {
                                tokens: vec![tokens[i].clone()],
                            },
                        );

                        let parameters = if param_text.is_empty() {
                            None
                        } else {
                            // Create parameters from the parameter text
                            let mut params = std::collections::HashMap::new();
                            params.insert("raw".to_string(), param_text.to_string());
                            // Preserve the verbatim label token as source for parameters
                            let param_tokens =
                                ScannerTokenSequence::from_tokens(vec![tokens[i].clone()]);
                            Some(HighLevelTokenBuilder::parameters_with_tokens(
                                params,
                                tokens[i].span().clone(),
                                param_tokens,
                            ))
                        };

                        (label_semantic_token, parameters)
                    } else {
                        // No colon found, treat as simple label
                        (
                            HighLevelTokenBuilder::text_span_with_tokens(
                                label_content.to_string(),
                                tokens[i].span().clone(),
                                ScannerTokenSequence {
                                    tokens: vec![tokens[i].clone()],
                                },
                            ),
                            None,
                        )
                    }
                } else {
                    // No colon found, treat as simple label
                    (
                        HighLevelTokenBuilder::text_span_with_tokens(
                            label_content.to_string(),
                            tokens[i].span().clone(),
                            ScannerTokenSequence {
                                tokens: vec![tokens[i].clone()],
                            },
                        ),
                        None,
                    )
                }
            } else {
                return Err(SemanticAnalysisError::AnalysisError(
                    "VerbatimBlock must end with VerbatimLabel".to_string(),
                ));
            }
        } else {
            return Err(SemanticAnalysisError::AnalysisError(
                "VerbatimBlock must have VerbatimLabel".to_string(),
            ));
        };

        // Aggregate all source tokens for preservation
        let aggregated_tokens = ScannerTokenSequence::from_tokens(tokens);

        // Transform to VerbatimBlock semantic token with aggregated tokens
        Ok(HighLevelTokenBuilder::verbatim_block_with_tokens(
            title_token,
            wall_token,
            content_token,
            label_token,
            parameters,
            wall_type,
            span,
            aggregated_tokens,
        ))
    }

    /// Find the position of the closing TxxtMarker in an annotation
    ///
    /// This helper method searches for the second TxxtMarker in an annotation,
    /// which marks the end of the label section.
    ///
    /// # Arguments
    /// * `tokens` - The scanner tokens to search
    ///
    /// # Returns
    /// * `Result<usize, SemanticAnalysisError>` - The position of the closing marker
    fn find_closing_txxt_marker(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<usize, SemanticAnalysisError> {
        let mut marker_count = 0;
        for (i, token) in tokens.iter().enumerate() {
            if matches!(token, ScannerToken::TxxtMarker { .. }) {
                marker_count += 1;
                if marker_count == 2 {
                    return Ok(i);
                }
            }
        }
        Err(SemanticAnalysisError::AnalysisError(
            "Annotation must have closing TxxtMarker".to_string(),
        ))
    }

    /// Parse label tokens and extract parameters if present
    ///
    /// This helper method processes label tokens and separates the main label
    /// from any parameters (key=value pairs).
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing the label (may include parameters)
    ///
    /// # Returns
    /// * `Result<(HighLevelToken, Option<HighLevelToken>), SemanticAnalysisError>` - (label, parameters)
    fn parse_label_with_parameters(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<(HighLevelToken, Option<HighLevelToken>), SemanticAnalysisError> {
        // Look for colon separator to identify parameters
        let colon_pos = tokens
            .iter()
            .position(|token| matches!(token, ScannerToken::Colon { .. }));

        if let Some(pos) = colon_pos {
            // Split into label and parameters
            let label_tokens = &tokens[..pos];
            let param_tokens = &tokens[pos + 1..];

            // Create label semantic token
            let label_token = self.tokens_to_text_span(label_tokens.to_vec())?;

            // Create parameters semantic token if there are parameter tokens
            let parameters = if param_tokens.is_empty()
                || param_tokens
                    .iter()
                    .all(|token| matches!(token, ScannerToken::Whitespace { .. }))
            {
                None
            } else {
                Some(self.parse_parameters(param_tokens)?)
            };

            Ok((label_token, parameters))
        } else {
            // No parameters, just create label
            let label_token = self.tokens_to_text_span(tokens.to_vec())?;
            Ok((label_token, None))
        }
    }

    /// Parse definition term tokens and extract parameters if present
    ///
    /// This helper method processes definition term tokens and separates the main term
    /// from any parameters (key=value pairs). Unlike labels, definition terms preserve
    /// whitespace to maintain formatting like verbatim titles.
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing the definition term (may include parameters)
    ///
    /// # Returns
    /// * `Result<(HighLevelToken, Option<HighLevelToken>), SemanticAnalysisError>` - (term, parameters)
    fn parse_definition_term_with_parameters(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<(HighLevelToken, Option<HighLevelToken>), SemanticAnalysisError> {
        // Look for colon separator to identify parameters
        let colon_pos = tokens
            .iter()
            .position(|token| matches!(token, ScannerToken::Colon { .. }));

        if let Some(pos) = colon_pos {
            // Split into term and parameters
            let term_tokens = &tokens[..pos];
            let param_tokens = &tokens[pos + 1..];

            // Create term semantic token (preserve whitespace)
            let term_token = self.tokens_to_text_span_preserve_whitespace(term_tokens)?;

            // Create parameters semantic token if there are parameter tokens
            let parameters = if param_tokens.is_empty()
                || param_tokens
                    .iter()
                    .all(|token| matches!(token, ScannerToken::Whitespace { .. }))
            {
                None
            } else {
                Some(self.parse_parameters(param_tokens)?)
            };

            Ok((term_token, parameters))
        } else {
            // No parameters, just create term (preserve whitespace)
            let term_token = self.tokens_to_text_span_preserve_whitespace(tokens)?;
            Ok((term_token, None))
        }
    }

    /// Parse parameter tokens into a Parameters semantic token
    ///
    /// This helper method processes parameter tokens and creates a structured
    /// Parameters semantic token with key-value pairs.
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing parameters
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The parameters semantic token
    fn parse_parameters(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Use the unified parameter builder from high-level tokens
        // This properly parses key=value pairs from scanner tokens
        HighLevelTokenBuilder::parameters_from_scanner_tokens(tokens).ok_or_else(|| {
            SemanticAnalysisError::InvalidParameterSyntax(
                "Failed to parse parameters from scanner tokens".to_string(),
            )
        })
    }

    /// Parse annotation content tokens into a semantic token
    ///
    /// This helper method processes content tokens after the closing TxxtMarker
    /// and creates an appropriate semantic token for the content.
    ///
    /// # Arguments
    /// * `tokens` - The tokens representing the annotation content
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The content semantic token
    fn parse_annotation_content(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        if tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Annotation content cannot be empty".to_string(),
            ));
        }

        // For content, preserve whitespace to maintain formatting
        // This will be enhanced in future iterations to handle complex content
        self.tokens_to_text_span_preserve_whitespace(tokens)
    }

    /// Convert a sequence of tokens to a TextSpan semantic token preserving whitespace
    ///
    /// This helper method combines multiple tokens into a single TextSpan,
    /// preserving whitespace for content that needs formatting.
    ///
    /// # Arguments
    /// * `tokens` - The tokens to combine
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The text span semantic token
    fn tokens_to_text_span_preserve_whitespace(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        if tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Cannot create text span from empty tokens".to_string(),
            ));
        }

        // Preserve whitespace for content but trim leading/trailing whitespace
        let content = tokens
            .iter()
            .map(|token| token.content())
            .collect::<Vec<&str>>()
            .join("")
            .trim()
            .to_string();

        let span = SourceSpan {
            start: tokens[0].span().start,
            end: tokens[tokens.len() - 1].span().end,
        };

        Ok(HighLevelTokenBuilder::text_span_with_tokens(
            content,
            span,
            ScannerTokenSequence {
                tokens: tokens.to_vec(),
            },
        ))
    }

    /// Convert a sequence of tokens to a TextSpan semantic token preserving exact whitespace
    /// It preserves all whitespace exactly as written (no trimming).
    fn tokens_to_text_span_exact(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        if tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Cannot create text span from empty tokens".to_string(),
            ));
        }

        // Preserve exact whitespace for verbatim content
        let content = tokens
            .iter()
            .map(|token| token.content())
            .collect::<Vec<&str>>()
            .join("");

        let span = SourceSpan {
            start: tokens[0].span().start,
            end: tokens[tokens.len() - 1].span().end,
        };

        Ok(HighLevelTokenBuilder::text_span_with_tokens(
            content,
            span,
            ScannerTokenSequence {
                tokens: tokens.to_vec(),
            },
        ))
    }

    /// This is a utility method to convert any scanner token to text content
    /// when we don't have a specific transformation for it yet.
    fn token_to_text_content(&self, token: &ScannerToken) -> String {
        match token {
            ScannerToken::Text { content, .. } => content.clone(),
            ScannerToken::Whitespace { content, .. } => content.clone(),
            ScannerToken::Newline { .. } => "\n".to_string(),
            ScannerToken::Dash { .. } => "-".to_string(),
            ScannerToken::Period { .. } => ".".to_string(),
            ScannerToken::LeftBracket { .. } => "[".to_string(),
            ScannerToken::RightBracket { .. } => "]".to_string(),
            ScannerToken::AtSign { .. } => "@".to_string(),
            ScannerToken::LeftParen { .. } => "(".to_string(),
            ScannerToken::RightParen { .. } => ")".to_string(),
            ScannerToken::Colon { .. } => ":".to_string(),
            ScannerToken::Equals { .. } => "=".to_string(),
            ScannerToken::Comma { .. } => ",".to_string(),
            ScannerToken::TxxtMarker { .. } => "::".to_string(),
            ScannerToken::Identifier { content, .. } => content.clone(),
            ScannerToken::SequenceMarker { marker_type, .. } => {
                // Convert sequence marker to its text representation
                marker_type.content().to_string()
            }
            _ => "".to_string(),
        }
    }
}

/// Errors that can occur during semantic analysis
#[derive(Debug)]
pub enum SemanticAnalysisError {
    /// Invalid token type encountered
    InvalidTokenType { expected: String, actual: String },
    /// General semantic analysis error
    AnalysisError(String),
}

impl std::fmt::Display for SemanticAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticAnalysisError::InvalidTokenType { expected, actual } => {
                write!(
                    f,
                    "Invalid token type: expected {}, got {}",
                    expected, actual
                )
            }
            SemanticAnalysisError::AnalysisError(msg) => {
                write!(f, "Semantic analysis error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SemanticAnalysisError {}
