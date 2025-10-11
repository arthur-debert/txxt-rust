# TXXT AST Architecture

This directory contains the complete AST (Abstract Syntax Tree) structure for the TXXT format, organized into focused modules for maintainability and clarity.

## Architecture Overview

The AST is designed to serve multiple tooling needs:
- **Language Server**: Character-precise hover, autocomplete, go-to-definition
- **Formatter**: Exact source reconstruction with whitespace control  
- **Linter**: Rich pattern matching and rule enforcement
- **Converter**: Pandoc-compatible structure for format interchange

## Key Design Principles

### 1. Token-Level Precision

Every text element maintains character-level token information to support advanced language server features. This enables:
- Hover information at exact cursor positions
- Autocomplete triggering within identifiers
- Precise syntax highlighting
- Character-accurate error underlining

### 2. Container Indentation Pattern

**Crucial insight**: The container is what gets indented, not the parent element.

```txxt
- Item 1.1                    // ListItem (level 0)
- Item 1.2                    // ListItem (level 0)  
  - Item 2.1                  // Container (level 1) -> List -> ListItem
  - Item 2.2                  // Container (level 1) -> List -> ListItem
```

This explains why flat lists don't need indentation - only nested content requires a Container.

### 3. Text Transform Layer

Every piece of text goes through a uniform transform layer:
- `Identity(Text("banana"))` for plain text
- `Emphasis(vec![Identity(Text("important"))])` for *important*  
- `Strong(vec![Emphasis(vec![Identity(Text("both"))])])` for **_both_**

### 4. Annotation Attachment System

Annotations are metadata that attaches to nodes based on proximity rules:
1. Document start → attach to document
2. Before element → attach to following element  
3. Last in level/group → attach to parent

### 5. Hierarchical Tree Structure

Levels are computed via tree traversal, never stored as attributes. This prevents synchronization issues and supports arbitrary nesting depth.

## Module Organization

### `base.rs`
Core document structure, metadata, and assembly information.

### `tokens.rs` 
Token-level precision with exact source positions for language server support.

### `structure.rs`
Containers, sessions, paragraphs, and hierarchical organization.

### `blocks.rs`
Block-level elements: verbatim, lists, definitions.

### `inlines.rs`
Inline elements with the text transform layer.

### `reference_types.rs`
Annotations, citations, cross-references, and metadata systems.

## List Styling - Critical Details

Lists and sessions support sophisticated styling schemes common in technical documents:

### Styling Rules

1. **Styling is a list attribute, not item attribute**
   - First item determines style for whole list
   - Mixed styling is preserved but not validated
   - Renderer can auto-correct if needed

2. **Markers are preserved exactly**
   - Parser saves actual input markers ("1.", "c)", "ii.", etc.)
   - No validation of sequence or correctness  
   - Enables flexible authoring and automated correction

3. **Forgiving parsing**
   - Mixed styles don't cause errors
   - Out-of-order numbering is accepted
   - Content is preserved, inconsistencies noted

### Supported Styles

- **Plain**: `-` for lists, no marker for sessions
- **Numerical**: `1.`, `2.`, `3.` 
- **Alphabetical**: `a)`, `b)`, `c)` or `A)`, `B)`, `C)`
- **Roman**: `i)`, `ii)`, `iii)` or `I)`, `II)`, `III)`

### Numbering Forms

- **Short form**: Each level shows only its number (`1.`, `a.`, `i.`)
- **Full form**: Each level shows complete hierarchy (`1.a.i.`)

### Examples

```txxt
1. Mom              // Numerical list, short form
2. Dad

a) Red              // Alphabetical list  
b) Blue

i) First            // Roman numeral list
ii) Second

3. Wrong            // Inconsistent but parsed fine
a) Mixed            // Different style but preserved

1. Format Specifications                    // Session with numerical style

    a) General Considerations               // Nested with alphabetical

        i) Character Encoding               // Nested with roman
        
1.a.i) Alternative Full Form               // Full form numbering
```

## Processing Pipeline

1. **Tokenization**: Source → Tokens with precise positions
2. **Parsing**: Tokens → Raw AST (annotations as regular blocks)
3. **Assembly**: Raw AST → Final Document (annotations properly attached)
4. **Tooling**: Document → Language Server/Formatter/Linter operations

The assembly phase is crucial for:
- Attaching annotations based on proximity rules
- Adding processing metadata (parser version, timestamps)
- Resolving cross-references and dependencies
- Computing tree statistics and validation

## Type Safety Benefits

Moving from the old `node_type: String` approach to proper Rust enums provides:
- Compile-time verification of AST structure
- Exhaustive pattern matching in visitors
- Clear separation of different node types
- Better IDE support and refactoring safety

## Future Extensions

The AST is designed for extensibility:
- `Custom` variants in major enums for user-defined elements
- Parameter maps for arbitrary metadata
- Flexible annotation system for tooling integration
- Pandoc-compatible structure for format conversion