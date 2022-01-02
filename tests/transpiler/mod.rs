use std::fs::{self, File};
use std::io::Write;

use surn::compiler::parser::Parser;
use surn::compiler::CompilerOptions;
use surn::transpiler::Transpiler;

pub const EXPRESSIONS: &str = "tests/resources/transpiler.surn";

#[test]
pub fn transpile_php() {
    let mut transpiler = Transpiler::new();
    transpiler.register_defaults();
    let contents = fs::read_to_string(EXPRESSIONS).unwrap();
    let mut parser = Parser::new(CompilerOptions::dev());
    let body = parser.parse_script(EXPRESSIONS.to_string(), contents);
    let code = transpiler
        .get("php")
        .unwrap()
        .generator
        .generate_to_string(body, CompilerOptions::dev());
    let mut f = File::create("tests/resources/test.php").unwrap();
    f.write_all(code.as_bytes()).unwrap();
}
