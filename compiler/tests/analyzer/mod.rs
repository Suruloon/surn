use std::fs;

use surn::compiler::lexer::analysis::analyze;
use surn::compiler::lexer::tokenizer::tokenize;

// Tests the tokenizer with the given file.
pub const TEST_A: &str = "tests/resources/test_a.surn";
pub const ERROR_A: &str = "tests/resources/error.debug";

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

// print the file contents
#[test]
pub fn test_print() {
    let contents = fs::read_to_string(ERROR_A).unwrap();
    println!("{}", contents);
}
