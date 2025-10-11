# TXXT Token Implementation Comparison

## Overview

This document compares the TXXT specification requirements (documented in `tokens.txxt`) with our current tokenizer implementation in `src/ast/tokens.rs` and `src/tokenizer/lexer.rs`.

## Implementation Status Summary

### ✅ **Fully Implemented** (3/15 token types)

| Token | Status | Implementation Quality | Test Coverage |
|-------|--------|----------------------|---------------|
| `SequenceMarker` | ✅ Complete | Excellent - all styles supported | 50 tests |
| `AnnotationMarker` | ✅ Complete | Excellent - full `::` syntax | 19 tests |
| `Text` | ✅ Complete | Good - basic alphanumeric content | 18 tests |

### 🟡 **Structurally Defined** (6/15 token types)

These tokens exist in the AST enum but lack recognition logic in the lexer:

| Token | AST Defined | Lexer Logic | Priority |
|-------|-------------|-------------|----------|
| `RefMarker` | ✅ | ❌ | **High** - Critical for links |
| `FootnoteNumber` | ✅ | ❌ | **High** - Subset of RefMarker |
| `VerbatimStart` | ✅ | ❌ | **High** - Core feature |
| `VerbatimContent` | ✅ | ❌ | **High** - Core feature |
| `Identifier` | ✅ | ✅ | **Medium** - Works but underused |
| `Dash` | ✅ | ❌ | **Low** - Rarely standalone |

### ❌ **Not Implemented** (6/15 token types)

| Token | Complexity | Priority | Notes |
|-------|------------|----------|-------|
| `Newline` | Low | **High** | Basic line structure |
| `BlankLine` | Low | **High** | Document structure |
| `Indent` | Medium | **High** | Essential for nesting |
| `Dedent` | Medium | **High** | Essential for nesting |
| Inline formatting | High | **Medium** | `*bold*`, `_italic_`, etc. |
| Citation markers | Medium | **Medium** | `[@ref]` syntax |

## Detailed Analysis

### Current Strengths

1. **Excellent sequence marker support**
   - All numbering styles: plain (`-`), numerical (`1.`), alphabetical (`a.`, `A.`), roman (`i.`, `I.`)
   - Proper validation: column 0 requirement, space after marker
   - Comprehensive edge case handling
   - 50 passing tests covering all variations

2. **Complete annotation marker implementation**
   - Correct `::` recognition with triple-colon rejection
   - Proper backtracking on failed matches
   - Good test coverage including edge cases
   - 19 passing tests

3. **Solid foundation architecture**
   - Type-safe AST with all token variants defined
   - Precise `SourceSpan` positioning for language server support
   - Proper tokenizer structure with backtracking
   - Good separation of concerns

### Critical Gaps

1. **Reference markers completely missing**
   ```rust
   // Missing from lexer.rs:
   fn read_ref_marker(&mut self) -> Option<Token> {
       // Should handle: [file.txxt], [#section], [@citation], [1]
   }
   ```

2. **No verbatim block support**
   ```rust
   // Missing from lexer.rs:
   fn read_verbatim_start(&mut self) -> Option<Token> {
       // Should handle: "title:" or ":"
   }
   ```

3. **No structural tokens**
   - No newline/blank line handling
   - No indentation tracking
   - Critical for document structure

### Specification vs Implementation Mismatches

1. **Token precedence order**
   - ✅ **Fixed**: Text now tried before Identifier (correct)
   - ✅ **Working**: Annotation markers tried first (correct)

2. **Missing inline formatting**
   - Spec requires: `*bold*`, `_italic_`, `` `code` ``, `#math#`
   - Implementation: None present

3. **Incomplete reference handling**
   - Spec requires: `[file]`, `[#section]`, `[@cite]`, `[1]`
   - Implementation: AST variant exists but no lexer logic

## Recommended Implementation Priority

### Phase 1: Core Structure (High Priority)
1. **Newline/BlankLine tokens** - Essential for document parsing
2. **Indent/Dedent tokens** - Required for proper nesting
3. **RefMarker recognition** - Critical for link functionality

### Phase 2: Content Features (Medium Priority)  
4. **VerbatimStart/VerbatimContent** - Major TXXT feature
5. **FootnoteNumber as RefMarker subset** - Common use case
6. **Basic inline formatting** - User-visible features

### Phase 3: Advanced Features (Lower Priority)
7. **Citation markers** - Specialized academic use
8. **Dash token** - Edge case handling
9. **Parameter parsing** - Complex syntax

## Test Coverage Analysis

Current test files:
- `tokenizer_sequence_marker_tests.rs` - 50 tests ✅
- `tokenizer_annotation_marker_tests.rs` - 19 tests ✅  
- `tokenizer_text_token_tests.rs` - 18 tests ✅

**Missing test files needed:**
- `tokenizer_ref_marker_tests.rs`
- `tokenizer_verbatim_tests.rs`
- `tokenizer_inline_formatting_tests.rs`
- `tokenizer_structural_tests.rs`

## Architecture Recommendations

1. **Extend lexer with missing token readers**
   ```rust
   // Add to lexer.rs tokenize() method:
   } else if let Some(token) = self.read_ref_marker() {
       tokens.push(token);
   } else if let Some(token) = self.read_verbatim_start() {
       tokens.push(token);
   ```

2. **Add structural token processing**
   ```rust
   // Handle newlines and indentation explicitly
   if ch == '\n' {
       tokens.push(self.handle_newline());
   }
   ```

3. **Implement proper inline formatting**
   ```rust
   // Try inline formatting before text
   } else if let Some(token) = self.read_inline_formatting() {
       tokens.push(token);
   ```

## Conclusion

The current implementation provides an excellent foundation with robust sequence marker and annotation marker support. The architecture is sound and extensible. Priority should be on implementing the missing structural tokens (newlines, indentation) and reference markers to achieve a functional TXXT parser that can handle the most common use cases.

The test-driven approach with rstest and proptest has proven effective and should continue for the remaining token types.