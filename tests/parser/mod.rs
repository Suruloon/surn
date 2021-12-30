// various tests to test the parser.use std::fs;

use std::{
    fs::{self, File},
    io::Write,
};

use surn::{parser::Parser, CompilerOptions};

pub const FULL_TEST: &str = "tests/resources/test_a.surn";
pub const EXPRESSIONS: &str = "tests/resources/expressions.surn";

#[test]
pub fn test_parse() {
    let contents = fs::read_to_string(FULL_TEST).unwrap();
    let mut parser = Parser::new(CompilerOptions::dev());
    let body = parser.parse_script("tests/parser/test.surn".to_string(), contents);
    let mut f = File::create("tests/resources/test.surn.ast").unwrap();
    f.write_all(format!("{:#?}", body).as_bytes()).unwrap();
}

#[test]
pub fn test_parse_expressions() {
    let contents = fs::read_to_string(EXPRESSIONS).unwrap();
    let mut parser = Parser::new(CompilerOptions::dev());
    let body = parser.parse_script("tests/parser/test.surn".to_string(), contents);
    let mut f = File::create("tests/resources/test.surn.ast").unwrap();
    f.write_all(format!("{:#?}", body).as_bytes()).unwrap();
}
