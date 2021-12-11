use std::{ops::Range, str::Chars};

/// Keeps a cache of the source buffer for the given context.
/// You can clear this using drop or `clean` on the struct.
#[derive(Clone, Debug)]
pub struct SourceBuffer {
    pub(crate) source: String,
}

pub struct SourceLine {
    offset: usize,
    len: usize,
    line: usize,
    source: String,
}

impl SourceLine {
    pub fn new(offset: usize, len: usize, line: usize, source: String) -> Self {
        Self {
            offset,
            len,
            line,
            source,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl SourceBuffer {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    pub fn empty() -> Self {
        Self {
            source: String::new(),
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

    pub fn get_lines(&self) -> Vec<SourceLine> {
        let mut lines = Vec::new();
        let mut line_start = 0;
        let mut line_end = 0;
        let mut line: usize = 1;
        for (i, c) in self.chars().enumerate() {
            if c == '\n' {
                lines.push(SourceLine {
                    offset: line_start,
                    len: line_end - line_start,
                    source: self.get(line_start..line_end),
                    line,
                });
                line += 1;
                line_start = i + 1;
                line_end = line_start;
            } else {
                line_end += 1;
            }
        }
        lines.push(SourceLine {
            offset: line_start,
            len: line_end - line_start,
            source: self.get(line_start..line_end),
            line,
        });
        lines
    }

    /// Attempts to find the line at the given offset and returns the entire line.
    /// Returns None if the offset is out of bounds.
    /// eg:
    /// ```rust no_run
    /// use util::source::SourceBuffer;
    /// let source = "var test = 10;\nvar apple = 4;".to_string();
    /// let buffer = SourceBuffer::new(&source);
    /// buffer.get_line(4); // returns: "var test = 10;"
    /// ```
    pub fn get_line_at(&self, offset: usize) -> Option<SourceLine> {
        self.get_lines()
            .into_iter()
            .find(|line| line.offset <= offset && offset <= line.offset + line.len)
    }
}
