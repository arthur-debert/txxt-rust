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
use crate::syntax::list_detection;

/// Parsed components of an annotation
///
/// Pure data structure representing the extracted parts of an annotation.
/// Used by `parse_annotation_components` for testable annotation parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationComponents {
    /// Position of the opening TxxtMarker (::)
    pub opening_marker_pos: usize,
    /// Position of the closing TxxtMarker (::)
    pub closing_marker_pos: usize,
    /// Start position of label tokens (after opening marker and whitespace)
    pub label_start: usize,
    /// End position of label tokens (before closing marker)
    pub label_end: usize,
    /// Start position of content tokens (after closing marker), if any
    pub content_start: Option<usize>,
}

/// Parsing context for context-aware element detection
///
/// After grammar simplification (issue #139), element types are distinguished by:
/// - Colon presence/absence at end of line
/// - Blank line presence/absence before content
/// - Indentation presence/absence after title
///
/// However, some elements (like sessions) are only valid in specific containers.
/// ParseContext tracks where we are in the document structure to enable
/// context-aware detection.
///
/// # Context Rules
///
/// - **Sessions**: Only allowed in DocumentRoot and SessionContainer
/// - **Definitions**: Allowed in any container (but not inside lists for now)
/// - **Paragraphs**: Allowed in ContentContainer and ListContent
/// - **Lists**: Allowed in ContentContainer and ListContent
///
/// # Unambiguous Patterns
///
/// Within allowed contexts, patterns are unambiguous:
/// - `Title\n\n<indent>` → Session (blank line + indent)
/// - `Term:\n<indent>` → Definition (colon + immediate indent, no blank)
/// - `Text\n<indent>` → Paragraph continuation
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseContext {
    /// Top-level document context
    /// Allows: Sessions, Definitions, Paragraphs, Lists, Annotations
    DocumentRoot,

    /// Inside a session container
    /// Allows: Sessions (nested), Definitions, Paragraphs, Lists, Annotations
    SessionContainer,

    /// Inside a definition or other content container
    /// Allows: Definitions, Paragraphs, Lists, Annotations
    /// Does NOT allow: Sessions
    ContentContainer,

    /// Inside a list item
    /// Allows: Paragraphs, Lists (nested), Annotations
    /// Does NOT allow: Sessions, Definitions (for now)
    ListContent,
}

impl ParseContext {
    /// Check if sessions are allowed in this context
    ///
    /// Sessions can only appear at document root or inside other sessions.
    /// They cannot appear in definitions, paragraphs, or list items.
    pub fn allows_sessions(self) -> bool {
        matches!(
            self,
            ParseContext::DocumentRoot | ParseContext::SessionContainer
        )
    }

    /// Check if definitions are allowed in this context
    ///
    /// Definitions are allowed everywhere except inside list items (for now).
    pub fn allows_definitions(self) -> bool {
        !matches!(self, ParseContext::ListContent)
    }

    /// Check if paragraphs are allowed in this context
    ///
    /// Paragraphs are allowed in all contexts.
    pub fn allows_paragraphs(self) -> bool {
        true
    }

    /// Check if lists are allowed in this context
    ///
    /// Lists are allowed in all contexts.
    pub fn allows_lists(self) -> bool {
        true
    }

    /// Get the context for parsing content inside a session
    pub fn session_content_context(self) -> ParseContext {
        ParseContext::SessionContainer
    }

    /// Get the context for parsing content inside a definition
    pub fn definition_content_context(self) -> ParseContext {
        ParseContext::ContentContainer
    }

    /// Get the context for parsing content inside a list item
    pub fn list_item_content_context(self) -> ParseContext {
        ParseContext::ListContent
    }
}

/// Parse annotation token structure into components
///
/// Pure function that validates annotation structure and extracts component positions.
/// This enables unit testing of annotation parsing logic without SemanticAnalyzer.
///
/// After grammar simplification (issue #139), annotations have the form:
/// - `:: label ::` - label only
/// - `:: label params ::` - label with parameters (whitespace separator)
/// - `:: params ::` - params-only
/// - `:: label :: content` - annotation with content after closing marker
///
/// # Arguments
/// * `tokens` - Scanner tokens representing the annotation
///
/// # Returns
/// * `Result<AnnotationComponents, String>` - Parsed component positions or error
///
/// # Examples
/// ```text
/// For input: `:: meta version=2.0 ::`
/// Returns: AnnotationComponents {
///   opening_marker_pos: 0,
///   closing_marker_pos: 4,
///   label_start: 2,
///   label_end: 4,
///   content_start: None,
/// }
/// ```
pub fn parse_annotation_components(
    tokens: &[ScannerToken],
) -> Result<AnnotationComponents, String> {
    // Validate minimum annotation structure: :: label ::
    if tokens.len() < 5 {
        return Err("Annotation must have at least 5 tokens: :: label ::".to_string());
    }

    // Check for opening TxxtMarker
    if !matches!(tokens[0], ScannerToken::TxxtMarker { .. }) {
        return Err("Annotation must start with TxxtMarker".to_string());
    }

    // Find closing TxxtMarker (second :: marker)
    let closing_marker_pos = find_closing_marker(tokens)?;
    if closing_marker_pos < 4 {
        return Err("Annotation must have proper structure: :: label ::".to_string());
    }

    // Label tokens are between opening marker+whitespace and closing marker
    // Skip position 1 if it's whitespace
    let label_start = if matches!(tokens.get(1), Some(ScannerToken::Whitespace { .. })) {
        2
    } else {
        1
    };

    let label_end = closing_marker_pos;

    // Content starts after closing marker, if there are more tokens
    let content_start = if closing_marker_pos + 1 < tokens.len() {
        Some(closing_marker_pos + 1)
    } else {
        None
    };

    Ok(AnnotationComponents {
        opening_marker_pos: 0,
        closing_marker_pos,
        label_start,
        label_end,
        content_start,
    })
}

/// Find the position of the closing TxxtMarker (second ::) in annotation
///
/// Pure helper function for finding the second TxxtMarker in a token stream.
///
/// # Arguments
/// * `tokens` - Scanner tokens to search
///
/// # Returns
/// * `Result<usize, String>` - Position of closing marker or error
fn find_closing_marker(tokens: &[ScannerToken]) -> Result<usize, String> {
    let mut marker_count = 0;
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token, ScannerToken::TxxtMarker { .. }) {
            marker_count += 1;
            if marker_count == 2 {
                return Ok(i);
            }
        }
    }
    Err("Annotation must have closing TxxtMarker".to_string())
}

/// Detect if a line pattern represents a session start (context-aware)
///
/// After grammar simplification (issue #139), sessions are identified by:
/// - Title line (text without colon at end)
/// - Followed by blank line
/// - Followed by indented content
///
/// This pattern is ONLY valid in contexts that allow sessions (DocumentRoot, SessionContainer).
/// In other contexts, the same pattern would be interpreted as a paragraph.
///
/// # Arguments
/// * `current_line` - Tokens for the current line
/// * `next_line` - Tokens for the next line (should be blank for session)
/// * `line_after_next` - Tokens for the line after the blank line (should be indented)
/// * `context` - Current parsing context
///
/// # Returns
/// * `true` if this is a session start pattern AND context allows sessions
///
/// # Examples
/// ```text
/// In DocumentRoot context:
///   Session Title
///   <blank>
///       Content
/// → Returns true (session allowed in DocumentRoot)
///
/// In ContentContainer context:
///   Same pattern
/// → Returns false (sessions not allowed in ContentContainer)
/// ```
pub fn is_session_start(
    current_line: &[ScannerToken],
    next_line: Option<&[ScannerToken]>,
    line_after_next: Option<&[ScannerToken]>,
    context: ParseContext,
) -> bool {
    // Sessions only allowed in specific contexts
    if !context.allows_sessions() {
        return false;
    }

    // Check current line is not a definition (no colon at end)
    if crate::syntax::is_definition_marker(current_line) {
        return false;
    }

    // Check current line has content (not blank)
    if crate::syntax::is_blank_line(current_line) {
        return false;
    }

    // Check next line is blank
    if !next_line.is_some_and(crate::syntax::is_blank_line) {
        return false;
    }

    // Check line after blank has indented content
    // (This is a simplified check - full implementation would check indent level)
    if let Some(tokens) = line_after_next {
        if !tokens.is_empty() {
            // In real implementation, would check for Indent token or leading whitespace
            return true;
        }
    }

    false
}

/// Detect if a line pattern represents a definition start (context-aware)
///
/// After grammar simplification (issue #139), definitions are identified by:
/// - Term line ending with single colon (:)
/// - Followed immediately by indented content (no blank line)
///
/// This pattern is valid in most contexts except ListContent (for now).
///
/// # Arguments
/// * `current_line` - Tokens for the current line (should end with colon)
/// * `next_line` - Tokens for the next line (should be indented, not blank)
/// * `context` - Current parsing context
///
/// # Returns
/// * `true` if this is a definition start pattern AND context allows definitions
///
/// # Examples
/// ```text
/// Term:
///     Definition content
/// → Returns true if context allows definitions
///
/// Inside ListContent:
///     Term:
///         Content
/// → Returns false (definitions not allowed in lists yet)
/// ```
pub fn is_definition_start(
    current_line: &[ScannerToken],
    next_line: Option<&[ScannerToken]>,
    context: ParseContext,
) -> bool {
    // Definitions only allowed in specific contexts
    if !context.allows_definitions() {
        return false;
    }

    // Check current line ends with colon (definition marker)
    if !crate::syntax::is_definition_marker(current_line) {
        return false;
    }

    // Check next line is NOT blank (immediate content required)
    if next_line.is_none_or(crate::syntax::is_blank_line) {
        return false;
    }

    // Check next line has content (is indented)
    // (In real implementation, would verify indent level)
    next_line.is_some_and(|tokens| !tokens.is_empty())
}

/// Detect if a line pattern represents a paragraph continuation
///
/// Paragraphs are identified by:
/// - Text line (no special markers)
/// - Optionally followed by indented content (continuation)
///
/// This is the "default" interpretation when more specific patterns don't match.
///
/// # Arguments
/// * `current_line` - Tokens for the current line
/// * `context` - Current parsing context
///
/// # Returns
/// * `true` if this line could be a paragraph (context allows paragraphs)
pub fn is_paragraph_line(current_line: &[ScannerToken], context: ParseContext) -> bool {
    // Paragraphs allowed in all contexts
    if !context.allows_paragraphs() {
        return false;
    }

    // Not a blank line
    if crate::syntax::is_blank_line(current_line) {
        return false;
    }

    // Not a definition marker
    if crate::syntax::is_definition_marker(current_line) {
        return false;
    }

    // Has some content
    !current_line.is_empty()
}

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
                ScannerToken::Whitespace { .. }
                    if pending_indentation.is_empty()
                        && crate::syntax::indentation_analysis::extract_leading_whitespace_from_tokens(
                            &scanner_tokens,
                            i,
                        )
                        .is_some() =>
                {
                    // Extract and capture the leading whitespace
                    pending_indentation =
                        crate::syntax::indentation_analysis::extract_leading_whitespace_from_tokens(
                            &scanner_tokens,
                            i,
                        )
                        .unwrap(); // Safe because we checked is_some() in guard
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

            // Check for verbatim block pattern (Issue #132)
            // Pattern: VerbatimBlockStart → (VerbatimContentLine | BlankLine)* → VerbatimBlockEnd
            ScannerToken::VerbatimBlockStart { .. } => {
                if let Some((tokens, consumed)) =
                    self.recognize_verbatim_block_pattern(scanner_tokens, start_index)?
                {
                    return Ok(Some((tokens, consumed)));
                }
            }

            _ => {}
        }

        Ok(None)
    }

    /// Recognize definition pattern (NEW syntax after grammar simplification):
    /// Pattern: Text + Colon + Newline + Whitespace + content
    /// Example: "Term:\n    Definition content"
    fn recognize_definition_pattern(
        &self,
        scanner_tokens: &[ScannerToken],
        start_index: usize,
    ) -> Result<Option<(Vec<ScannerToken>, usize)>, SemanticAnalysisError> {
        // Need at least: Text, Colon, Newline, Whitespace, Text
        if start_index + 4 >= scanner_tokens.len() {
            return Ok(None);
        }

        // Must start with Text
        if !matches!(scanner_tokens[start_index], ScannerToken::Text { .. }) {
            return Ok(None);
        }

        // Collect term tokens until we hit Colon
        let mut i = start_index;
        let mut term_tokens = vec![];

        // Collect term (can be multiple Text tokens with whitespace)
        while i < scanner_tokens.len() {
            match &scanner_tokens[i] {
                ScannerToken::Text { .. } => {
                    term_tokens.push(scanner_tokens[i].clone());
                    i += 1;
                }
                ScannerToken::Whitespace { .. } => {
                    term_tokens.push(scanner_tokens[i].clone());
                    i += 1;
                }
                ScannerToken::Colon { .. } => {
                    // Found the colon - check if this is a definition pattern
                    break;
                }
                _ => {
                    // Not a definition pattern
                    return Ok(None);
                }
            }
        }

        // Must have found a Colon
        if i >= scanner_tokens.len() || !matches!(scanner_tokens[i], ScannerToken::Colon { .. }) {
            return Ok(None);
        }

        let colon_index = i;
        i += 1; // Move past colon

        // Next must be Newline
        if i >= scanner_tokens.len() || !matches!(scanner_tokens[i], ScannerToken::Newline { .. }) {
            return Ok(None);
        }
        i += 1; // Move past newline

        // Next must be Indent token (NOT just Whitespace!)
        // This ensures content is at a DEEPER indentation level than the subject line.
        // If both lines are at the same level (e.g., "Key concepts:" followed by list at same indent),
        // there won't be an Indent token, so this is not a definition.
        if i >= scanner_tokens.len() || !matches!(scanner_tokens[i], ScannerToken::Indent { .. }) {
            return Ok(None);
        }
        i += 1; // Move past Indent

        // After Indent, there should be Whitespace and then content
        // Skip whitespace if present
        if i < scanner_tokens.len() && matches!(scanner_tokens[i], ScannerToken::Whitespace { .. })
        {
            i += 1;
        }

        // Next must be content (Text or other content token)
        if i >= scanner_tokens.len() {
            return Ok(None);
        }

        // Valid definition pattern found!
        // Return ONLY the subject line tokens (term + colon + newline)
        // The content will be processed separately as Indent + PlainTextLine tokens
        let mut subject_tokens = term_tokens;
        subject_tokens.push(scanner_tokens[colon_index].clone()); // Colon
        subject_tokens.push(scanner_tokens[colon_index + 1].clone()); // Newline

        // Consume only the subject line (term + colon + newline)
        let consumed = colon_index + 2 - start_index;

        Ok(Some((subject_tokens, consumed)))
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

        // Look for pattern: TxxtMarker + Whitespace + (Identifier|Text) + ... + TxxtMarker
        // The "..." can include parameters (Colon, Identifier, Equals, Text, Comma, etc.)
        if !matches!(
            scanner_tokens[start_index + 1],
            ScannerToken::Whitespace { .. }
        ) {
            return Ok(None);
        }

        if !matches!(
            scanner_tokens[start_index + 2],
            ScannerToken::Identifier { .. } | ScannerToken::Text { .. }
        ) {
            return Ok(None);
        }

        // Scan forward to find closing TxxtMarker
        let mut closing_marker_pos = None;
        for (i, token) in scanner_tokens.iter().enumerate().skip(start_index + 3) {
            if matches!(token, ScannerToken::TxxtMarker { .. }) {
                closing_marker_pos = Some(i);
                break;
            }
            // Stop if we hit structural tokens before finding closing marker
            if matches!(
                token,
                ScannerToken::Newline { .. }
                    | ScannerToken::BlankLine { .. }
                    | ScannerToken::Indent { .. }
                    | ScannerToken::Dedent { .. }
            ) {
                break;
            }
        }

        if let Some(closing_pos) = closing_marker_pos {
            // Extract tokens up to and including the closing TxxtMarker
            let mut consumed = closing_pos - start_index + 1;

            // Look for optional content after the closing TxxtMarker
            let mut i = closing_pos + 1;
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
            Ok(Some((pattern_tokens, consumed)))
        } else {
            // No closing TxxtMarker found
            Ok(None)
        }
    }

    /// Recognize verbatim block pattern (Issue #132)
    /// Pattern: VerbatimBlockStart → (VerbatimContentLine | BlankLine)* → VerbatimBlockEnd
    fn recognize_verbatim_block_pattern(
        &self,
        scanner_tokens: &[ScannerToken],
        start_index: usize,
    ) -> Result<Option<(Vec<ScannerToken>, usize)>, SemanticAnalysisError> {
        if start_index >= scanner_tokens.len() {
            return Ok(None);
        }

        // Must start with VerbatimBlockStart
        if !matches!(
            scanner_tokens[start_index],
            ScannerToken::VerbatimBlockStart { .. }
        ) {
            return Ok(None);
        }

        let mut tokens = vec![scanner_tokens[start_index].clone()];
        let mut i = start_index + 1;

        // Collect content lines (VerbatimContentLine and BlankLine tokens)
        while i < scanner_tokens.len() {
            match &scanner_tokens[i] {
                ScannerToken::VerbatimContentLine { .. } | ScannerToken::BlankLine { .. } => {
                    tokens.push(scanner_tokens[i].clone());
                    i += 1;
                }
                ScannerToken::VerbatimBlockEnd { .. } => {
                    // Found the end marker
                    tokens.push(scanner_tokens[i].clone());
                    i += 1;
                    let consumed = i - start_index;
                    return Ok(Some((tokens, consumed)));
                }
                _ => {
                    // Unexpected token - not a valid verbatim block
                    return Err(SemanticAnalysisError::AnalysisError(format!(
                        "Unexpected token in verbatim block: {:?}",
                        scanner_tokens[i]
                    )));
                }
            }
        }

        // Reached end without finding VerbatimBlockEnd
        Err(SemanticAnalysisError::AnalysisError(
            "Unterminated verbatim block".to_string(),
        ))
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
            // Definition pattern: Text + Colon + Newline (subject line only)
            // (new multi-line syntax after grammar simplification)
            // Content is processed separately as Indent + PlainTextLine tokens
            ScannerToken::Text { .. } => {
                // Check if pattern contains a Colon (definition marker)
                let has_colon = pattern_tokens
                    .iter()
                    .any(|t| matches!(t, ScannerToken::Colon { .. }));
                let has_newline = pattern_tokens
                    .iter()
                    .any(|t| matches!(t, ScannerToken::Newline { .. }));
                if has_colon && has_newline && pattern_tokens.len() >= 2 {
                    // Definition subject line: term + colon (+ optional newline)
                    let span = SourceSpan {
                        start: pattern_tokens[0].span().start,
                        end: pattern_tokens[pattern_tokens.len() - 1].span().end,
                    };
                    return self.transform_definition(pattern_tokens, span);
                }
            }

            // Annotation pattern: TxxtMarker + Whitespace + (Identifier|Text) + ... + TxxtMarker
            // The middle can include parameters: Colon + Identifier + Equals + Text/QuotedString + Comma + ...
            ScannerToken::TxxtMarker { .. } => {
                if pattern_tokens.len() >= 5
                    && matches!(pattern_tokens[1], ScannerToken::Whitespace { .. })
                    && matches!(
                        pattern_tokens[2],
                        ScannerToken::Identifier { .. } | ScannerToken::Text { .. }
                    )
                {
                    // Find closing TxxtMarker
                    let has_closing_marker = pattern_tokens
                        .iter()
                        .skip(3)
                        .any(|t| matches!(t, ScannerToken::TxxtMarker { .. }));

                    if has_closing_marker {
                        let span = SourceSpan {
                            start: pattern_tokens[0].span().start,
                            end: pattern_tokens[pattern_tokens.len() - 1].span().end,
                        };
                        return self.transform_annotation(pattern_tokens, span);
                    }
                }
            }

            // NEW: Verbatim block pattern (Issue #132)
            // Pattern: VerbatimBlockStart → (VerbatimContentLine | BlankLine)* → VerbatimBlockEnd
            ScannerToken::VerbatimBlockStart { .. } => {
                if pattern_tokens.len() >= 2
                    && matches!(
                        pattern_tokens[pattern_tokens.len() - 1],
                        ScannerToken::VerbatimBlockEnd { .. }
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
        let (style, form) = list_detection::classify_sequence_marker(marker_type);

        // Convert to high-level types
        let hl_style = match style {
            list_detection::NumberingStyle::Plain => HighLevelNumberingStyle::Plain,
            list_detection::NumberingStyle::Numerical => HighLevelNumberingStyle::Numeric,
            list_detection::NumberingStyle::Alphabetical => HighLevelNumberingStyle::Alphabetic,
            list_detection::NumberingStyle::Roman => HighLevelNumberingStyle::Roman,
        };

        let hl_form = match form {
            list_detection::NumberingForm::Regular => HighLevelNumberingForm::Regular,
            list_detection::NumberingForm::Extended => HighLevelNumberingForm::Extended,
        };

        (hl_style, hl_form)
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
        // Use pure function to parse annotation structure
        let components =
            parse_annotation_components(&tokens).map_err(SemanticAnalysisError::AnalysisError)?;

        // Extract label tokens (between opening and closing markers)
        let label_tokens = &tokens[components.label_start..components.label_end];
        let (label_token, parameters) = self.parse_label_with_parameters(label_tokens)?;

        // Extract content tokens (after closing marker, if any)
        let content = if let Some(content_start) = components.content_start {
            let content_tokens = &tokens[content_start..];
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
    /// This implements the Definition transformation after grammar simplification.
    /// Definition tokens represent structured elements that define terms, concepts,
    /// and entities. They follow the new simplified pattern:
    /// Text + Colon (parameters come from optional trailing annotations)
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
        // NEW multi-line definition structure: term + : + newline + indent + content
        // The Definition token represents just the subject line (term:)
        // Content becomes separate tokens that AST construction will group

        // Find the Colon position
        let colon_pos = tokens
            .iter()
            .position(|t| matches!(t, ScannerToken::Colon { .. }))
            .ok_or_else(|| {
                SemanticAnalysisError::AnalysisError(
                    "Definition pattern must contain Colon".to_string(),
                )
            })?;

        // Extract term (everything before Colon)
        let term_tokens = &tokens[..colon_pos];
        if term_tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Definition must have a term before the colon".to_string(),
            ));
        }

        let term_token = self.tokens_to_text_span_preserve_whitespace(term_tokens)?;

        // For span, use just the subject line (term + colon + newline)
        // Find newline position after colon
        let newline_pos = tokens
            .iter()
            .skip(colon_pos + 1)
            .position(|t| matches!(t, ScannerToken::Newline { .. }))
            .map(|p| p + colon_pos + 1);

        let subject_line_span = if let Some(nl_pos) = newline_pos {
            SourceSpan {
                start: tokens[0].span().start,
                end: tokens[nl_pos].span().end,
            }
        } else {
            span.clone()
        };

        // Aggregate only the subject line tokens for the Definition token
        let subject_tokens = if let Some(nl_pos) = newline_pos {
            &tokens[..=nl_pos]
        } else {
            &tokens[..=colon_pos]
        };
        let aggregated_tokens = ScannerTokenSequence::from_tokens(subject_tokens.to_vec());

        // Transform to Definition semantic token
        // Parameters are None - they come from optional trailing annotations
        Ok(HighLevelTokenBuilder::definition_with_tokens(
            term_token,
            None,
            subject_line_span,
            aggregated_tokens,
        ))
    }

    /// Transform scanner tokens into a VerbatimBlock semantic token
    ///
    /// This implements the VerbatimBlock transformation as specified in Issue #89.
    /// VerbatimBlock tokens represent content that preserves exact formatting and spacing
    /// using the wall architecture pattern:
    /// title + wall + content (IgnoreLine/BlankLine) + label + parameters
    ///
    /// # Arguments
    /// * `tokens` - Vector of scanner tokens that form a verbatim block
    /// * `span` - The source span covering the entire verbatim block
    ///
    /// # Returns
    /// * `Result<HighLevelToken, SemanticAnalysisError>` - The semantic token
    ///
    /// Transform verbatim block pattern into VerbatimBlock semantic token (Issue #132)
    ///
    /// Pattern: VerbatimBlockStart → (VerbatimContentLine | BlankLine)* → VerbatimBlockEnd
    pub fn transform_verbatim_block(
        &self,
        tokens: Vec<ScannerToken>,
        span: SourceSpan,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        if tokens.len() < 2 {
            return Err(SemanticAnalysisError::AnalysisError(
                "VerbatimBlock must have at least VerbatimBlockStart and VerbatimBlockEnd"
                    .to_string(),
            ));
        }

        // Extract start token
        let (title, wall_type) = if let ScannerToken::VerbatimBlockStart {
            title, wall_type, ..
        } = &tokens[0]
        {
            (title.clone(), wall_type.clone())
        } else {
            return Err(SemanticAnalysisError::AnalysisError(
                "VerbatimBlock must start with VerbatimBlockStart".to_string(),
            ));
        };

        // Extract end token
        let label_raw =
            if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = &tokens[tokens.len() - 1] {
                label_raw.clone()
            } else {
                return Err(SemanticAnalysisError::AnalysisError(
                    "VerbatimBlock must end with VerbatimBlockEnd".to_string(),
                ));
            };

        // Create title token
        let title_token = HighLevelTokenBuilder::text_span_with_tokens(
            title.clone(),
            tokens[0].span().clone(),
            ScannerTokenSequence {
                tokens: vec![tokens[0].clone()],
            },
        );

        // Create wall token (placeholder for high-level token structure)
        let wall_token = HighLevelTokenBuilder::text_span_with_tokens(
            String::new(),
            tokens[0].span().clone(),
            ScannerTokenSequence {
                tokens: vec![tokens[0].clone()],
            },
        );

        // Create Vec<HighLevelToken> from VerbatimContentLine and BlankLine scanner tokens
        // Each scanner token becomes a proper high-level token (IgnoreLine or BlankLine)
        // Content preserves full indentation+content; wall-stripping happens at AST level
        let mut content_lines = Vec::new();
        for token in &tokens[1..tokens.len() - 1] {
            match token {
                ScannerToken::VerbatimContentLine {
                    content,
                    indentation,
                    span,
                } => {
                    // Create IgnoreLine with full indentation + content preserved
                    // AST layer will handle wall-stripping based on wall_type
                    content_lines.push(HighLevelToken::IgnoreLine {
                        content: format!("{}{}", indentation, content),
                        span: span.clone(),
                        tokens: ScannerTokenSequence {
                            tokens: vec![token.clone()],
                        },
                    });
                }
                ScannerToken::BlankLine { span, .. } => {
                    content_lines.push(HighLevelToken::BlankLine {
                        span: span.clone(),
                        tokens: ScannerTokenSequence {
                            tokens: vec![token.clone()],
                        },
                    });
                }
                _ => {
                    // Ignore other tokens (shouldn't happen in well-formed verbatim blocks)
                }
            }
        }

        // Parse label and parameters from label_raw using UNIFIED parser
        //
        // ARCHITECTURE: Verbatim vs Annotations
        // - Verbatim scanner: Produces VerbatimBlockEnd { label_raw: "python:version=3.11" }
        //   → Semantic layer: Already has string, calls unified parser directly
        //
        // - Annotation scanner: Produces [Identifier("python"), Colon, Identifier("version"), ...]
        //   → Semantic layer: Extracts string from tokens, calls unified parser
        //
        // Both converge at parse_label_and_parameters_from_string() for:
        // - Namespace-aware label validation (e.g., "org.example.python")
        // - Unified parameter parsing
        // - Consistent Label + Parameters token creation
        //
        // See: parse_label_and_parameters_from_string() for unified implementation
        let (label, parameters) = self.parse_label_and_parameters_from_string(
            &label_raw,
            tokens[tokens.len() - 1].span().clone(),
        )?;

        // Aggregate all source tokens
        let aggregated_tokens = ScannerTokenSequence::from_tokens(tokens);

        // Build VerbatimBlock high-level token with Vec<HighLevelToken> content
        Ok(HighLevelTokenBuilder::verbatim_block_with_tokens(
            title_token,
            wall_token,
            content_lines,
            label,
            parameters,
            wall_type,
            span,
            aggregated_tokens,
        ))
    }

    /// **UNIFIED LABEL PARSING** - Used by both annotations and verbatim blocks
    ///
    /// Parse a label string (with optional parameters) into validated Label and Parameters tokens.
    /// This is the single source of truth for label+parameter parsing across the semantic layer.
    ///
    /// # Architecture Decision
    ///
    /// The scanner layer intentionally produces different formats:
    /// - **Annotations**: Fine-grained tokens [TxxtMarker, Identifier, Colon, ...]
    ///   - Scanner: `:: python:version=3.11 ::` → individual tokens
    ///   - Semantic: Extracts string from tokens → calls this method
    ///
    /// - **Verbatim**: Pre-combined string in VerbatimBlockEnd
    ///   - Scanner: `:: python:version=3.11` → label_raw: "python:version=3.11"
    ///   - Semantic: Already has string → calls this method directly
    ///
    /// Both converge at this unified method for consistent validation and Label creation.
    ///
    /// # Format
    /// - Simple label: `"python"` → Label token, no parameters
    /// - With parameters: `"python:version=3.11,syntax=true"` → Label + Parameters tokens
    ///
    /// # Arguments
    /// * `label_raw` - Raw label string (may include `:params` suffix)
    /// * `base_span` - Source span for the entire label_raw string
    ///
    /// # Returns
    /// * `Result<(Label token, Optional Parameters token), Error>`
    ///
    /// # See Also
    /// - Scanner tokens: `src/cst/scanner_tokens.rs` - VerbatimBlockEnd, Identifier
    /// - Label validation: `src/syntax/elements/components/label.rs` - parse_label()
    /// - Parameter parsing: `src/cst/parameter_scanner.rs` - scan_parameter_string()
    fn parse_label_and_parameters_from_string(
        &self,
        label_raw: &str,
        base_span: SourceSpan,
    ) -> Result<(HighLevelToken, Option<HighLevelToken>), SemanticAnalysisError> {
        // After grammar simplification (issue #139), annotations use:
        // - `:: label params ::` - label followed by params (no colon separator)
        // - `:: params ::` - params-only (no label)
        //
        // Strategy:
        // 1. Check if entire string looks like parameters (contains '=' before any whitespace)
        // 2. If yes: params-only annotation, create dummy label
        // 3. If no: split on first whitespace, first word is label, rest is params

        let trimmed = label_raw.trim();

        // Check if this is a params-only annotation
        // Params start with key=value pattern (identifier followed by '=')
        let is_params_only = trimmed.chars().next().is_some_and(|c| {
            // If starts with letter/digit and contains '=' before whitespace
            c.is_alphanumeric() && {
                let first_whitespace = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
                let equals_pos = trimmed.find('=').unwrap_or(trimmed.len());
                equals_pos < first_whitespace
            }
        });

        if is_params_only {
            // Params-only annotation: entire string is parameters
            let param_start = base_span.start;

            // Use unified parameter scanner from Issue #135
            let param_scanner_tokens =
                crate::cst::parameter_scanner::scan_parameter_string(trimmed, param_start);

            // Convert scanner tokens to high-level Parameters token
            let parameters = crate::cst::high_level_tokens::HighLevelTokenBuilder::parameters_from_scanner_tokens(
                &param_scanner_tokens,
            );

            // Create empty label for params-only annotations
            let label_token = crate::cst::high_level_tokens::HighLevelToken::Label {
                text: String::new(),
                span: base_span.clone(),
                tokens: crate::cst::ScannerTokenSequence::new(),
            };

            Ok((label_token, parameters))
        } else {
            // Split on first whitespace to separate label from parameters
            if let Some(whitespace_pos) = trimmed.find(char::is_whitespace) {
                let label_part = &trimmed[..whitespace_pos];
                let params_part = trimmed[whitespace_pos..].trim();

                // Calculate span for label portion only
                let label_span = SourceSpan {
                    start: base_span.start,
                    end: Position {
                        row: base_span.start.row,
                        column: base_span.start.column + whitespace_pos,
                    },
                };

                // Create validated Label token with namespace support
                let label_token = self.create_validated_label(label_part, label_span)?;

                // Parse parameters using unified parameter scanner
                let parameters = if params_part.is_empty() {
                    None
                } else {
                    let param_start = Position {
                        row: base_span.start.row,
                        column: base_span.start.column + whitespace_pos + 1,
                    };

                    // Use unified parameter scanner from Issue #135
                    let param_scanner_tokens = crate::cst::parameter_scanner::scan_parameter_string(
                        params_part,
                        param_start,
                    );

                    // Convert scanner tokens to high-level Parameters token
                    crate::cst::high_level_tokens::HighLevelTokenBuilder::parameters_from_scanner_tokens(
                        &param_scanner_tokens,
                    )
                };

                Ok((label_token, parameters))
            } else {
                // No whitespace, entire string is label, no parameters
                let label_token = self.create_validated_label(trimmed, base_span)?;
                Ok((label_token, None))
            }
        }
    }

    /// Create a validated Label token with namespace support
    ///
    /// This helper validates label syntax including:
    /// - Must start with letter (a-z, A-Z)
    /// - Valid characters: letters, digits, underscore, dash
    /// - Namespace separator: period (.) - e.g., "org.example.python"
    /// - No consecutive periods, no leading/trailing periods
    ///
    /// # Arguments
    /// * `label_text` - The label string to validate
    /// * `span` - Source span for this label
    ///
    /// # Returns
    /// * `Result<HighLevelToken::Label, Error>`
    ///
    /// # See Also
    /// - `src/syntax/elements/components/label.rs` - validate_label(), parse_label()
    fn create_validated_label(
        &self,
        label_text: &str,
        span: SourceSpan,
    ) -> Result<HighLevelToken, SemanticAnalysisError> {
        // Use namespace-aware label validation from Issue #124
        crate::syntax::elements::components::label::validate_label(label_text)
            .map_err(|e| SemanticAnalysisError::AnalysisError(format!("Invalid label: {}", e)))?;

        // Create Label token (not TextSpan)
        Ok(crate::cst::high_level_tokens::HighLevelToken::Label {
            text: label_text.to_string(),
            span,
            tokens: crate::cst::ScannerTokenSequence::new(),
        })
    }

    /// Parse label tokens and extract parameters if present (ANNOTATIONS)
    ///
    /// This method is used by annotations to convert scanner tokens into label+parameters.
    /// It extracts a string from the tokens, then delegates to the unified
    /// `parse_label_and_parameters_from_string()` method.
    ///
    /// # Architecture
    /// - Annotations receive fine-grained scanner tokens
    /// - This method extracts string representation
    /// - Delegates to unified string-based parser for consistent validation
    ///
    /// # Arguments
    /// * `tokens` - Scanner tokens representing the label (may include Colon and parameter tokens)
    ///
    /// # Returns
    /// * `Result<(Label token, Optional Parameters token), Error>`
    ///
    /// # See Also
    /// - Unified parser: `parse_label_and_parameters_from_string()`
    /// - Verbatim parsing: `transform_verbatim_block()` - uses same unified parser
    fn parse_label_with_parameters(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<(HighLevelToken, Option<HighLevelToken>), SemanticAnalysisError> {
        if tokens.is_empty() {
            return Err(SemanticAnalysisError::AnalysisError(
                "Cannot parse label from empty tokens".to_string(),
            ));
        }

        // Extract string from tokens (PRESERVE whitespace for new grammar)
        // After grammar simplification (issue #139), whitespace separates label from params
        let mut label_raw = String::new();
        for token in tokens {
            label_raw.push_str(token.content());
        }

        // Calculate span from first to last token
        let span = SourceSpan {
            start: tokens[0].span().start,
            end: tokens[tokens.len() - 1].span().end,
        };

        // Delegate to unified string-based parser
        // This ensures consistent label validation and parameter parsing
        // for both annotations and verbatim blocks
        self.parse_label_and_parameters_from_string(&label_raw, span)
    }
    // NOTE: parse_definition_term_with_parameters() and parse_parameters() removed
    // After grammar simplification, definitions no longer have inline parameters.
    // Parameters come from optional trailing annotations in AST construction.

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
    #[allow(dead_code)]
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
    /// Invalid parameter syntax
    InvalidParameterSyntax(String),
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
            SemanticAnalysisError::InvalidParameterSyntax(msg) => {
                write!(f, "Invalid parameter syntax: {}", msg)
            }
            SemanticAnalysisError::AnalysisError(msg) => {
                write!(f, "Semantic analysis error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SemanticAnalysisError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_txxt_marker(col: usize) -> ScannerToken {
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + 2,
                },
            },
        }
    }

    fn make_text(content: &str, col: usize) -> ScannerToken {
        ScannerToken::Text {
            content: content.to_string(),
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + content.len(),
                },
            },
        }
    }

    fn make_whitespace(col: usize) -> ScannerToken {
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + 1,
                },
            },
        }
    }

    #[test]
    fn test_parse_annotation_components_simple() {
        // Input: `:: label ::`
        let tokens = vec![
            make_txxt_marker(0),   // ::
            make_whitespace(2),    // (space)
            make_text("label", 3), // label
            make_whitespace(8),    // (space)
            make_txxt_marker(9),   // ::
        ];

        let result = parse_annotation_components(&tokens);
        assert!(result.is_ok());

        let components = result.unwrap();
        assert_eq!(components.opening_marker_pos, 0);
        assert_eq!(components.closing_marker_pos, 4);
        assert_eq!(components.label_start, 2); // After opening marker and whitespace
        assert_eq!(components.label_end, 4); // Before closing marker
        assert_eq!(components.content_start, None);
    }

    #[test]
    fn test_parse_annotation_components_with_content() {
        // Input: `:: label :: some content`
        let tokens = vec![
            make_txxt_marker(0),      // ::
            make_whitespace(2),       // (space)
            make_text("label", 3),    // label
            make_whitespace(8),       // (space)
            make_txxt_marker(9),      // ::
            make_whitespace(11),      // (space)
            make_text("content", 12), // content
        ];

        let result = parse_annotation_components(&tokens);
        assert!(result.is_ok());

        let components = result.unwrap();
        assert_eq!(components.opening_marker_pos, 0);
        assert_eq!(components.closing_marker_pos, 4);
        assert_eq!(components.label_start, 2);
        assert_eq!(components.label_end, 4);
        assert_eq!(components.content_start, Some(5)); // After closing marker
    }

    #[test]
    fn test_parse_annotation_components_too_short() {
        // Input too short: only 3 tokens
        let tokens = vec![make_txxt_marker(0), make_whitespace(2), make_text("x", 3)];

        let result = parse_annotation_components(&tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must have at least 5 tokens"));
    }

    #[test]
    fn test_parse_annotation_components_missing_opening_marker() {
        // Input doesn't start with TxxtMarker
        let tokens = vec![
            make_text("label", 0),
            make_whitespace(5),
            make_txxt_marker(6),
            make_whitespace(8),
            make_txxt_marker(9),
        ];

        let result = parse_annotation_components(&tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must start with TxxtMarker"));
    }

    #[test]
    fn test_parse_annotation_components_missing_closing_marker() {
        // Input has only one TxxtMarker
        let tokens = vec![
            make_txxt_marker(0),
            make_whitespace(2),
            make_text("label", 3),
            make_whitespace(8),
            make_text("more", 9),
        ];

        let result = parse_annotation_components(&tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("closing TxxtMarker"));
    }

    // Context-aware detection tests

    fn make_colon(col: usize) -> ScannerToken {
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + 1,
                },
            },
        }
    }

    fn make_blank_line() -> Vec<ScannerToken> {
        vec![make_whitespace(0)]
    }

    fn make_indented_line() -> Vec<ScannerToken> {
        vec![
            make_whitespace(0),
            make_whitespace(1),
            make_text("content", 2),
        ]
    }

    #[test]
    fn test_parse_context_allows_sessions() {
        assert!(ParseContext::DocumentRoot.allows_sessions());
        assert!(ParseContext::SessionContainer.allows_sessions());
        assert!(!ParseContext::ContentContainer.allows_sessions());
        assert!(!ParseContext::ListContent.allows_sessions());
    }

    #[test]
    fn test_parse_context_allows_definitions() {
        assert!(ParseContext::DocumentRoot.allows_definitions());
        assert!(ParseContext::SessionContainer.allows_definitions());
        assert!(ParseContext::ContentContainer.allows_definitions());
        assert!(!ParseContext::ListContent.allows_definitions());
    }

    #[test]
    fn test_is_session_start_in_document_root() {
        let current = vec![make_text("Session Title", 0)];
        let next = Some(make_blank_line());
        let after_next = Some(make_indented_line());

        assert!(is_session_start(
            &current,
            next.as_deref(),
            after_next.as_deref(),
            ParseContext::DocumentRoot
        ));
    }

    #[test]
    fn test_is_session_start_in_content_container() {
        // Same pattern but wrong context - should fail
        let current = vec![make_text("Title", 0)];
        let next = Some(make_blank_line());
        let after_next = Some(make_indented_line());

        assert!(!is_session_start(
            &current,
            next.as_deref(),
            after_next.as_deref(),
            ParseContext::ContentContainer
        ));
    }

    #[test]
    fn test_is_session_start_no_blank_line() {
        // Missing blank line - should fail
        let current = vec![make_text("Title", 0)];
        let next = Some(make_indented_line()); // Not blank!
        let after_next = Some(make_indented_line());

        assert!(!is_session_start(
            &current,
            next.as_deref(),
            after_next.as_deref(),
            ParseContext::DocumentRoot
        ));
    }

    #[test]
    fn test_is_definition_start_with_colon() {
        let current = vec![make_text("Term", 0), make_colon(4)];
        let next = Some(make_indented_line());

        assert!(is_definition_start(
            &current,
            next.as_deref(),
            ParseContext::DocumentRoot
        ));
    }

    #[test]
    fn test_is_definition_start_in_list_content() {
        // Definitions not allowed in list content
        let current = vec![make_text("Term", 0), make_colon(4)];
        let next = Some(make_indented_line());

        assert!(!is_definition_start(
            &current,
            next.as_deref(),
            ParseContext::ListContent
        ));
    }

    #[test]
    fn test_is_definition_start_with_blank_line() {
        // Definition requires immediate content (no blank line)
        let current = vec![make_text("Term", 0), make_colon(4)];
        let next = Some(make_blank_line());

        assert!(!is_definition_start(
            &current,
            next.as_deref(),
            ParseContext::DocumentRoot
        ));
    }

    #[test]
    fn test_is_paragraph_line_simple() {
        let line = vec![make_text("Some text", 0)];

        assert!(is_paragraph_line(&line, ParseContext::DocumentRoot));
        assert!(is_paragraph_line(&line, ParseContext::ContentContainer));
    }

    #[test]
    fn test_is_paragraph_line_with_definition_marker() {
        // Line ending with colon is not a paragraph
        let line = vec![make_text("Term", 0), make_colon(4)];

        assert!(!is_paragraph_line(&line, ParseContext::DocumentRoot));
    }

    #[test]
    fn test_is_paragraph_line_blank() {
        // Blank lines are not paragraphs
        let line = make_blank_line();

        assert!(!is_paragraph_line(&line, ParseContext::DocumentRoot));
    }
}
