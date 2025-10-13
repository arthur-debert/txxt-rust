# TXXT Testing Infrastructure

## TxxtCorpora: Specification-Driven Test Case Extraction

The `TxxtCorpora` utility provides a robust, spec-driven testing framework that extracts test cases directly from the authoritative specification documents in `docs/specs/`. This ensures perfect alignment between documentation and tests.

### Quick Start

```rust
use tests::corpora::{TxxtCorpora, ProcessingStage};

// Load a test case (Raw text by default)
let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")?;
assert_eq!(corpus.source_text, "This is a basic paragraph...");

// Load with tokenization 
let corpus = TxxtCorpora::load_with_processing(
    "txxt.core.spec.paragraph.valid.simple", 
    ProcessingStage::Tokens
)?;
let tokens = corpus.tokens().unwrap();

// Load all test cases
let all_corpora = TxxtCorpora::load_all()?;
```

### Test Case Syntax in Specification Documents

Test cases are embedded in specification documents using labeled verbatim blocks:

```txxt
Simple paragraph example:
    This is a basic paragraph containing plain text.
:: txxt.core.spec.paragraph.valid.simple ::

Error case with parameters:
    - This is not a valid list.
:: txxt.core.spec.list.error.singleItem:error="ParseError",message="Lists require multiple items",line=1,column=1 ::
```

### Processing Stages

The `ProcessingStage` enum allows testing at different pipeline stages:

- `Raw` (default) - Extracted text as-is from specifications
- `Tokens` - Tokenized stream from the tokenizer
- `BlockedTokens` - Block-grouped tokens after lexical analysis  
- `ParsedAst` - Parsed AST structure
- `FullDocument` - Full document with all processing complete

### Integration Points for Parser Developers

**When writing tokenizer tests:**
```rust
let corpus = TxxtCorpora::load_with_processing(
    "txxt.core.spec.paragraph.valid.simple",
    ProcessingStage::Tokens
)?;
// Use corpus.tokens() to get expected tokenization
```

**When writing parser tests:**
```rust
let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")?;
let ast = your_parser::parse(&corpus.source_text)?;
insta::assert_yaml_snapshot!(ast);
```

**When writing error handling tests:**
```rust
let corpus = TxxtCorpora::load("txxt.core.spec.list.error.singleItem")?;
assert!(corpus.is_error_case());
assert_eq!(corpus.expected_error(), Some("ParseError"));

let result = your_parser::parse(&corpus.source_text);
assert!(result.is_err());
// Verify error matches expected error from corpus.parameters
```

### Adding New Test Cases

1. **In specification documents** (`docs/specs/elements/*.txxt`):
   ```txxt
   Your example description:
       The actual TXXT content to test
   :: txxt.core.spec.element.category.name ::
   ```

2. **Test naming convention**: `txxt.core.spec.{element}.{category}.{name}`
   - `element`: paragraph, list, session, etc.
   - `category`: valid, error, edge
   - `name`: descriptive identifier

3. **Error cases**: Add parameters to the label:
   ```txxt
   :: txxt.core.spec.list.error.singleItem:error="ParseError",message="Lists require multiple items" ::
   ```

### Integration with Real Pipeline Components

The corpora system is designed for easy integration:

```rust
// TODO: Replace placeholder with real tokenizer
fn apply_tokenization(text: &str) -> Vec<Token> {
    crate::tokenizer::tokenize(text) // Your actual tokenizer
}

// TODO: Replace placeholder with real parser  
fn apply_parsing(text: &str) -> AST {
    crate::parser::parse(text) // Your actual parser
}
```

Update `tests/corpora.rs` to integrate with actual pipeline components as they become available.

### Best Practices

1. **Always add test cases to specifications**: Don't create isolated test strings
2. **Use meaningful names**: Make test case identifiers descriptive
3. **Test both success and error cases**: Include expected error parameters
4. **Use appropriate processing stages**: Test at the right pipeline level
5. **Leverage snapshot testing**: Use `insta` for complex output validation

### File Locations

- **Core utility**: `tests/corpora.rs`
- **Integration tests**: `tests/parser_integration.rs` 
- **Test cases**: Embedded in `docs/specs/elements/*.txxt`
- **Documentation**: This file (`tests/README.md`)

This infrastructure ensures that:
- ✅ Specification and tests stay in sync
- ✅ No test cases are missed or forgotten
- ✅ Changes to specs immediately update tests
- ✅ Parser development follows spec-driven approach
- ✅ All pipeline stages can be systematically tested