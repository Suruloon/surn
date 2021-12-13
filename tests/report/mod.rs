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
            34..49,
            "String contents can not contain this message.".to_string(),
            Some("Illegal message".into()),
        )
        .set_message("This is a test.".to_string())
        .print();
}
