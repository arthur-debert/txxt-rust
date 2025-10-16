//! Debug test for detokenizer issues

use txxt::lexer::tokenize;
use txxt::tools::detokenizer::Detokenizer;

#[test]
fn debug_math_expression() {
    let input = "#E = mc^2#";

    let tokens = tokenize(input);
    println!("Tokens: {:?}", tokens);

    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer.detokenize_for_verification(&tokens).unwrap();
    println!("Reconstructed: {}", reconstructed);

    assert_eq!(input, reconstructed);
}

#[test]
fn debug_annotation_with_params() {
    let input = ":: warning:severity=high :: Critical security information";

    let tokens = tokenize(input);
    println!("\nAnnotation tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {:2}: {:?}", i, token);
    }

    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer.detokenize_for_verification(&tokens).unwrap();
    println!("\nReconstructed: {}", reconstructed);

    assert_eq!(input, reconstructed);
}

#[test]
fn debug_verbatim_label_with_params() {
    let input = ":: python:version=3.11,style=functional";

    let tokens = tokenize(input);
    println!("\nVerbatim label tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {:2}: {:?}", i, token);
    }

    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer.detokenize_for_verification(&tokens).unwrap();
    println!("\nReconstructed: {}", reconstructed);

    assert_eq!(input, reconstructed);
}
