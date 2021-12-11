use std::{str::Chars, ops::Range};

/// Keeps a cache of the source buffer for the given context.
/// You can clear this using drop or `clean` on the struct.
pub struct SourceBuffer {
    pub(crate) source: String
}

impl SourceBuffer {
    pub fn new(source: String) -> Self {
        Self {
            source
        }
    }

    /// Gets a range of the source buffer.
    /// eg:
    /// ```rust no_run
    /// use std::ops::Range;
    /// use util::source::SourceBuffer;
    /// let source = "hello world".to_string();
    /// let buffer = SourceBuffer::new(&source);
    /// buffer.get(0..5);
    /// ```
    pub fn get(&self, rng: Range<usize>) -> String {
        let mut result = String::new();
        for i in rng {
            result.push(self.chars().nth(i).unwrap());
        }
        result
    }

    pub fn chars(&self) -> Chars {
        self.source.chars()
    }
}