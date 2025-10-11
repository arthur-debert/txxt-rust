//! TXXT Tokenizer
//!
//! This module implements the lexical analysis phase of the TXXT parser, converting raw TXXT text into a stream of tokens.
//!
//! ## Architecture
//!
//! The tokenizer is structured into three main components:
//!
//! ### 1. [`tokens`] - Token Definitions
//! - Defines all token types (e.g., `Text`, `Indent`, `SequenceMarker`, etc.)
//! - Provides the [`Token`] struct with position information
//! - Maps closely to the Python reference implementation
//!
//! ### 2. [`verbatim_scanner`] - Verbatim Block Detection
//! - **Pass 0**: Pre-processes the input to identify verbatim blocks
//! - Verbatim blocks are regions where normal parsing rules don't apply
//! - Started by lines ending with `:` and ended by `(label)` patterns
//! - Essential for handling code blocks and other literal content
//!
//! ### 3. [`lexer`] - Main Tokenization Logic
//! - **Pass 1**: Processes the input line by line
//! - Handles indentation-based structure (INDENT/DEDENT tokens)
//! - Recognizes various syntax elements:
//!   - List markers (`1.`, `-`, etc.)
//!   - Pragma annotations (`:: label ::`)
//!   - Definitions (`Term ::`)
//!   - Inline formatting (`*bold*`, `_italic_`, etc.)
//!   - References (`[link]`, `[@citation]`, `[42]`)
//!
//! ## Key Features
//!
//! ### Indentation Sensitivity
//! - Tracks indentation levels with an indent stack
//! - Emits INDENT tokens when indentation increases
//! - Emits DEDENT tokens when indentation decreases
//! - Handles tab-to-space conversion (1 tab = 4 spaces)
//!
//! ### Verbatim Block Handling
//! ```text
//! Code example:
//!     console.log("Hello, world!");
//!     // This content is preserved exactly
//! (javascript)
//! ```
//!
//! ### List Recognition
//! ```text
//! 1. Ordered list item
//! 2. Another item
//!     - Nested unordered item
//!     a) Alphabetic numbering
//! ```
//!
//! ### Inline Formatting
//! ```text
//! Text with *bold*, _italic_, `code`, and #math# formatting.
//! ```
//!
//! ### References and Citations
//! ```text
//! See [external link] or [@academic-citation] or footnote [42].
//! ```
//!
//! ### Pragma Annotations
//! ```text
//! :: title :: Document Title
//! :: author :: Author Name
//! :: metadata :: key=value, quoted="string value"
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use txxt::tokenizer::{tokenize, TokenType};
//!
//! let text = ":: title :: My Document\n\nThis is a *bold* statement.";
//! let tokens = tokenize(text);
//!
//! for token in tokens {
//!     println!("{:?}: {:?}", token.token_type, token.value);
//! }
//! ```
//!
//! ## Implementation Notes
//!
//! ### Character-by-Character Processing
//! For inline formatting, the lexer uses character-by-character processing rather than regex to:
//! - Handle nested brackets correctly
//! - Ensure formatting markers are adjacent to text
//! - Distinguish between different reference types in one pass
//! - Gracefully handle unclosed/unmatched markers
//!
//! ### Performance Considerations
//! - Pre-compiled regex patterns for common cases
//! - Two-pass approach (verbatim scanning + main tokenization)
//! - Minimal allocations during tokenization
//!
//! ### Compatibility
//! The Rust implementation aims for 100% compatibility with the Python reference implementation,
//! using the same token types and following the same parsing rules.

pub mod lexer;
pub mod tokens;
pub mod verbatim_scanner;

#[cfg(test)]
mod tests;

pub use lexer::Lexer;
pub use tokens::{Token, TokenType};
pub use verbatim_scanner::{VerbatimBlock, VerbatimScanner};

/// Tokenize TXXT text into a stream of tokens.
///
/// This is the main entry point for tokenization. It creates a [`Lexer`] instance
/// and processes the input text to produce a vector of [`Token`]s.
///
/// # Examples
///
/// ```rust
/// use txxt::tokenizer::{tokenize, TokenType};
///
/// let text = "Hello *world*!";
/// let tokens = tokenize(text);
///
/// assert_eq!(tokens[0].token_type, TokenType::Text);
/// assert_eq!(tokens[1].token_type, TokenType::StrongMarker);
/// ```
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(text);
    lexer.tokenize()
}
