use surn::{report::Report, util::source::SourceBuffer};

#[test]
pub fn test_snippet_print() {
    let code = "
    fn main() {
        println!(\"Hello, world!\");
    }
    ";
    Report::new()
        .set_source(SourceBuffer::new(code.to_string()))
        .make_snippet(5..7, "This keyword must be spelled out.".to_string(), None)
        .make_snippet(
            25..32,
            "This macro could not be found.".to_string(),
            Some("Try using print".into()),
        )
        .set_message("This is a test.".to_string())
        .print();
}
