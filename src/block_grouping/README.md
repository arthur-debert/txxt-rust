# TXXT Block Grouping

This module implements the block grouping stage of the TXXT parser, which organizes tokens into a hierarchical tree of semantic blocks with proper container structures.

## Architecture

The block grouping module consists of three main components:

### 1. `container_types.rs` - Type Definitions
- Defines `BlockType` enum for all semantic block types
- Defines `ContainerType` enum for content vs session containers
- Provides type checking methods for container compatibility

### 2. `blocks.rs` - Block Structures
- `BlockNode` - represents a semantic block with optional container
- `ContentContainer` - holds child blocks (cannot contain sessions)
- `SessionContainer` - holds child blocks (can contain sessions)
- `Block` enum - unified type for nodes and containers

### 3. `builder.rs` - Block Tree Construction
- `BlockTreeBuilder` - main algorithm for converting tokens to blocks
- Three-stage process: token tree → blank line splitting → semantic blocks
- Handles session detection, list grouping, and container assignment

## Key Features

### Container System
Following the TXXT specification, the module implements a uniform container system:

- **ContentContainer**: Used by annotations, definitions, lists
  - Cannot contain session blocks
  - Enforces +1 indentation for children
  
- **SessionContainer**: Used by document root and sessions
  - Can contain any block type including sessions
  - Enables hierarchical document structure

### Session Detection
The module implements advanced session detection based on:
- Blank line precedence (handled by tokenizer)
- Presence of indented children
- Exclusion of other block types (lists, annotations, etc.)

### Block Types
Supports all TXXT semantic elements:
- `Root` - document root with session container
- `Session` - section with title and session container
- `Paragraph` - text content without container
- `List` / `ListItem` - list structures with content containers
- `Definition` - term definitions with content containers
- `Annotation` - pragma-style annotations with content containers
- `Verbatim` - code/literal blocks without containers
- `TextLine` - individual text lines within paragraphs
- `BlankLine` - blank line tokens

## Usage

### Basic Block Grouping
```rust
use txxt::tokenizer::tokenize;
use txxt::block_grouping::build_block_tree;

let text = ":: title :: My Document\n\nThis is content.";
let tokens = tokenize(text);
let block_tree = build_block_tree(tokens);
```

### CLI Tool
The module includes a CLI tool for XML output verification:

```bash
cargo run --bin block_grouper input.txxt
cargo run --bin block_grouper tokens.xml
```

Output format:
```xml
<root>
  <source>input.txxt</source>
  <blocks>
    <block type="annotation">
      <label>title</label>
      <content-container indent-level="1">
        <block type="paragraph">
          <content>My Document</content>
        </block>
      </content-container>
    </block>
  </blocks>
</root>
```

## Integration

The block grouping stage fits between tokenization and final parsing:

1. **Tokenizer** → Stream of tokens with INDENT/DEDENT
2. **Block Grouping** → Hierarchical block tree with containers  
3. **Parser** → Final AST with semantic validation

This intermediate representation makes the final parsing stage much simpler by:
- Resolving structural ambiguities
- Establishing container relationships
- Organizing content into logical blocks

## Testing

The module includes comprehensive tests:
- Unit tests for individual block types
- Integration tests with real TXXT content
- Container compatibility validation
- Session detection edge cases

Run tests with:
```bash
cargo test block_grouping
```

## Implementation Notes

### Ambiguity Resolution
The module resolves TXXT's structural ambiguities:
- **Sessions vs Paragraphs**: Requires indented children
- **Lists vs Paragraphs**: Single items become paragraphs
- **Dialog Detection**: Uses end-punctuation patterns

### Performance
- Single-pass token processing
- Recursive tree construction
- Minimal memory allocation
- Efficient blank line splitting

### Future Enhancements
- Enhanced session detection heuristics
- List nesting validation
- Dialog detection implementation
- Performance optimizations for large documents