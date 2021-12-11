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
        .make_snippet(17.., "This keyword must be spelled out.".to_string(), None)
        .set_message("This is a test.".to_string())
        .print();
}
