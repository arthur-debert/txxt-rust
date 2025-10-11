pub mod lexer;
pub mod tokens;
pub mod verbatim_scanner;

#[cfg(test)]
mod tests;

pub use lexer::Lexer;
pub use tokens::{Token, TokenType};
pub use verbatim_scanner::{VerbatimBlock, VerbatimScanner};

pub fn tokenize(text: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(text);
    lexer.tokenize()
}
