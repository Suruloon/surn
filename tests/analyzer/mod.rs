use std::fs;

use surn::lexer::analysis::analyze;
use surn::lexer::tokenizer::tokenize;

// Tests the tokenizer with the given file.
pub const TEST_A: &str = "tests/analyzer/test_a.surn";

#[test]
pub fn test_analyze() {
    let contents = fs::read_to_string(TEST_A).unwrap();
    let tokens = tokenize(contents.as_str());

    let results = analyze(tokens);
    if results.is_ok() {
        panic!("Analyzer failed to detect errors.");
    } else {
        println!("{}", results.unwrap_err());
    }
}
