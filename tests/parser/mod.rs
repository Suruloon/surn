// various tests to test the parser.use std::fs;

use std::fs;

use surn::{parser::Parser, CompilerOptions};

pub const TEST: &str = "tests/parser/test.surn";

#[test]
pub fn test_parse() {
    let contents = fs::read_to_string(TEST).unwrap();
    let mut parser = Parser::new(CompilerOptions::dev());
    let body = parser.parse_script("Test Script".to_string(), contents);
    dbg!(body);
}
