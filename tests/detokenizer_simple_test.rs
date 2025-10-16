//! Simple debug test for detokenizer

use txxt::lexer::tokenize;
use txxt::tools::detokenizer::Detokenizer;

#[test]
fn debug_simple_text() {
    let original = "Hello, world!";

    // Step 1: Tokenize
    let tokens1 = tokenize(original);
    println!("Original: {:?}", original);
    println!("Tokens1 ({}):", tokens1.len());
    for (i, token) in tokens1.iter().enumerate() {
        println!("  [{}] {:?}", i, token);
    }

    // Step 2: Detokenize for verification
    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer.detokenize_for_verification(&tokens1).unwrap();
    println!("\nReconstructed: {:?}", reconstructed);

    // Step 4: Re-tokenize
    let tokens2 = tokenize(&reconstructed);
    println!("Tokens2 ({}):", tokens2.len());
    for (i, token) in tokens2.iter().enumerate() {
        println!("  [{}] {:?}", i, token);
    }

    // Check differences
    println!("\nToken count: {} vs {}", tokens1.len(), tokens2.len());
}

#[test]
fn debug_paragraph() {
    let original = "First line.\nSecond line.";

    let tokens1 = tokenize(original);
    println!("Original: {:?}", original);
    println!("Tokens1 ({}):", tokens1.len());
    for (i, token) in tokens1.iter().enumerate() {
        println!("  [{}] {:?}", i, token);
    }

    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer.detokenize_for_verification(&tokens1).unwrap();
    println!("\nReconstructed: {:?}", reconstructed);

    let tokens2 = tokenize(&reconstructed);
    println!("Tokens2 ({}):", tokens2.len());
    for (i, token) in tokens2.iter().enumerate() {
        println!("  [{}] {:?}", i, token);
    }
}
