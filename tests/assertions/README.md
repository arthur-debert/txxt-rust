# AST Assertion Framework

Ergonomic assertion helpers for validating parsed AST elements.

## Status

**Phase 1 Complete:**

- ✅ All Expected structs defined (`expected.rs`)
- ✅ Shared validators implemented (`validators.rs`)
- ✅ Reference implementation: `assert_paragraph()` (`mod.rs`)
- ✅ Framework tests (`tests.rs`)

**To Be Implemented:**

- ⏳ `assert_list()` - During Parser 2.1.2
- ⏳ `assert_definition()` - During Parser 2.1.3
- ⏳ Other elements - As needed

## Quick Example

```rust
use tests::assertions::{assert_paragraph, ParagraphExpected};

let para = parse_paragraph(&source).unwrap();

assert_paragraph(&para, ParagraphExpected {
    text_contains: Some("expected content"),
    has_formatting: Some(true),
    ..Default::default()
});
```

## Adding New Assertions

When implementing a parser element:

1. Expected struct already exists in `expected.rs`
2. Copy `assert_paragraph()` from `mod.rs` as template
3. Rename to `assert_your_element()`
4. Update downcast logic
5. Implement element-specific validation
6. Reuse shared validators
7. Add to exports in `mod.rs`

**Time: ~30 minutes per element**

## Design

- **One function per element type**
- **Optional validation** - Only `Some()` fields are checked
- **Shared logic** - Common validators prevent duplication
- **Helpful errors** - Clear expected vs actual messages

See `docs/dev/parser-core/Parser-1.3.1-Per-element-assertion-base.txxt` for complete design.
