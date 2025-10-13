use txxt::tokenizer::Lexer;

fn main() {
    // Create a custom debug lexer to trace the flow
    struct DebugLexer<'a> {
        lexer: &'a mut Lexer,
    }
    
    impl<'a> DebugLexer<'a> {
        fn tokenize_with_trace(&mut self) -> Vec<txxt::ast::tokens::Token> {
            let mut tokens = Vec::new();
            
            while self.lexer.position < self.lexer.input.len() {
                let pos = self.lexer.position;
                let ch = self.lexer.input.get(pos).copied();
                println!("Position {}: char {:?}", pos, ch);
                
                // Try each tokenizer in order (simplified version of the actual logic)
                if let Some(ch) = ch {
                    if ch == '\n' {
                        println!("  -> Would try newline");
                    } else if ch == ' ' || ch == '\t' {
                        println!("  -> Would try whitespace");
                    } else if ch == ':' {
                        println!("  -> Would try colon (for :: markers)");
                    } else if ch == '[' {
                        println!("  -> Would try references");
                    } else if ch == '@' {
                        println!("  -> Would try at-sign");
                    } else if ch == '*' || ch == '_' || ch == '`' || ch == '#' {
                        println!("  -> Would try inline delimiter");
                    } else {
                        println!("  -> Would try text");
                    }
                }
                
                // Advance to avoid infinite loop in this debug version
                self.lexer.position += 1;
            }
            
            tokens
        }
    }
    
    let input = "🎉_italic_";
    println!("=== Tokenizing: {:?} ===", input);
    
    let mut lexer = Lexer::new(input);
    let mut debug_lexer = DebugLexer { lexer: &mut lexer };
    debug_lexer.tokenize_with_trace();
}