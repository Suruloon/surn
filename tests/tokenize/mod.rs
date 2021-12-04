use std::fs;

use surn::lexer::{token::TokenType, tokenizer::tokenize};

// Tests the tokenizer with the given file.
pub const TEST_A: &str = "tests/tokenize/test_a.surn";

#[test]
pub fn test_tokenizer() {
    let code = fs::read_to_string(TEST_A).unwrap();

    let tokens = tokenize(code.as_str());
    for token in tokens {
        println!("{:?}", token);
    }
}

#[test]
pub fn test_token_comparison() {
    let one_token = tokenize("identifier").get(0).unwrap().clone();
    let two_token = tokenize("another_identifier").get(0).unwrap().clone();

    assert_eq!(one_token.kind(), two_token.kind());

    assert_ne!(TokenType::LeftBrace, TokenType::RightBrace);
}


#[test]
pub fn apple() {
}