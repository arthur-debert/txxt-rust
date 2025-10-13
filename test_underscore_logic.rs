fn main() {
    // Test the logic from read_text function
    let test_strings = vec![
        "🎉_italic_",
        "café_italic_",
        "test_italic_",
        "_italic_",
    ];
    
    for input in test_strings {
        println!("\n=== Testing: {:?} ===", input);
        let chars: Vec<char> = input.chars().collect();
        
        // Simulate read_text logic
        let mut pos = 0;
        let mut text_content = String::new();
        
        while pos < chars.len() {
            let ch = chars[pos];
            
            if ch == '_' {
                // Check next character (like in read_text)
                if let Some(&next_ch) = chars.get(pos + 1) {
                    println!("  At '_' (pos {}), next char is '{}' (alphanumeric={})", 
                             pos, next_ch, next_ch.is_alphanumeric());
                    
                    if next_ch.is_alphanumeric() {
                        println!("    -> Including '_' in text token");
                        text_content.push(ch);
                        pos += 1;
                    } else {
                        println!("    -> Stopping text token here");
                        break;
                    }
                } else {
                    println!("  At '_' (pos {}), no next char", pos);
                    break;
                }
            } else if ch.is_whitespace() || matches!(ch, '*' | '`' | '#' | '-' | '[' | ']' | '@' | ':' | '(' | ')' | '=' | ',' | '.') {
                println!("  At '{}' (pos {}) - special delimiter, stopping", ch, pos);
                break;
            } else {
                println!("  At '{}' (pos {}) - adding to text", ch, pos);
                text_content.push(ch);
                pos += 1;
            }
        }
        
        println!("  Result: text token = {:?}", text_content);
        
        // What happens with the test cases according to current logic:
        // "🎉_italic_" -> text="🎉_italic" (because _ is followed by 'i')
        // "café_italic_" -> text="café_italic" (because _ is followed by 'i')
        // "test_italic_" -> text="test_italic" (because _ is followed by 'i')
        // "_italic_" -> text="" (stops at first _ if it's at the beginning)
    }
}