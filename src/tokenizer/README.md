# TXXT Tokenizer

This module implements the lexical analysis phase of the TXXT parser, converting raw TXXT text into a stream of tokens.

## Architecture

The tokenizer is structured into three main components:

### 1. `tokens.rs` - Token Definitions
- Defines all token types (e.g., `Text`, `Indent`, `SequenceMarker`, etc.)
- Provides the `Token` struct with position information
- Maps closely to the Python reference implementation

### 2. `verbatim_scanner.rs` - Verbatim Block Detection
- **Pass 0**: Pre-processes the input to identify verbatim blocks
- Verbatim blocks are regions where normal parsing rules don't apply
- Started by lines ending with `:` and ended by `(label)` patterns
- Essential for handling code blocks and other literal content

### 3. `lexer.rs` - Main Tokenization Logic
- **Pass 1**: Processes the input line by line
- Handles indentation-based structure (INDENT/DEDENT tokens)
- Recognizes various syntax elements:
  - List markers (`1.`, `-`, etc.)
  - Pragma annotations (`:: label ::`)
  - Definitions (`Term ::`)
  - Inline formatting (`*bold*`, `_italic_`, etc.)
  - References (`[link]`, `[@citation]`, `[42]`)

## Key Features

### Indentation Sensitivity
- Tracks indentation levels with an indent stack
- Emits INDENT tokens when indentation increases
- Emits DEDENT tokens when indentation decreases
- Handles tab-to-space conversion (1 tab = 4 spaces)

### Verbatim Block Handling
```txxt
Code example:
    console.log("Hello, world!");
    // This content is preserved exactly
(javascript)
```

### List Recognition
```txxt
1. Ordered list item
2. Another item
    - Nested unordered item
    a) Alphabetic numbering
```

### Inline Formatting
```txxt
Text with *bold*, _italic_, `code`, and #math# formatting.
```

### References and Citations
```txxt
See [external link] or [@academic-citation] or footnote [42].
```

### Pragma Annotations
```txxt
:: title :: Document Title
:: author :: Author Name
:: metadata :: key=value, quoted="string value"
```

## Usage

```rust
use txxt::tokenizer::{tokenize, TokenType};

let text = ":: title :: My Document\n\nThis is a *bold* statement.";
let tokens = tokenize(text);

for token in tokens {
    println!("{:?}: {:?}", token.token_type, token.value);
}
```

## Testing

The tokenizer includes comprehensive tests:

1. **Unit Tests** (`tests.rs`) - Test individual features
2. **Integration Tests** (`tests/tokenizer_tests.rs`) - Test against reference files
3. **Reference Validation** - Uses `txxt-documents-clean/` for validation

The test fixtures automatically load `.txxt` files and their corresponding `.tokens.xml` files to verify that our implementation matches the expected output.

## Implementation Notes

### Character-by-Character Processing
For inline formatting, the lexer uses character-by-character processing rather than regex to:
- Handle nested brackets correctly
- Ensure formatting markers are adjacent to text
- Distinguish between different reference types in one pass
- Gracefully handle unclosed/unmatched markers

### Performance Considerations
- Pre-compiled regex patterns for common cases
- Two-pass approach (verbatim scanning + main tokenization)
- Minimal allocations during tokenization

### Compatibility
The Rust implementation aims for 100% compatibility with the Python reference implementation, using the same token types and following the same parsing rules.