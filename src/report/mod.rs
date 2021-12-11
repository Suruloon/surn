use std::fmt;

use crate::util::source::SourceBuffer;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReportKind {
    Error,
    Warning,
    Notice,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub code: u64,
    pub message: String,
    pub source: SourceBuffer,
    pub snippets: Vec<Snippet>,
}

impl Report {
    pub fn new() -> Self {
        Report {
            code: 0,
            message: String::new(),
            source: SourceBuffer::empty(),
            snippets: Vec::new(),
        }
    }

    pub fn set_code(mut self, code: u64) -> Self {
        self.code = code;
        self
    }

    pub fn set_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn set_source(mut self, source: SourceBuffer) -> Self {
        self.source = source;
        self
    }

    pub fn empty_snippet(&self) -> Snippet {
        let snip = Snippet::empty().set_source(self.source.clone()).clone();
        snip
    }

    pub fn add_snippet(mut self, snippet: Snippet) -> Self {
        self.snippets.push(snippet);
        self
    }
}

/// A snippet will produce the "line" of code that is being reported on.
/// EG:
/// ```readme no_run
/// 2  |  Source Code
///    |  Start~~~End Message
/// ```
#[derive(Clone, Debug)]
pub struct Snippet {
    pub(crate) message: String,
    source: SourceBuffer,
    start: usize,
    end: usize,
    line: usize,
    multiline: bool,
    padding: usize,
}

impl Snippet {
    pub fn new(
        source: SourceBuffer,
        message: String,
        start: usize,
        end: usize,
        line: usize,
    ) -> Self {
        Snippet {
            message: message,
            source,
            start,
            end,
            line,
            multiline: false,
            padding: 0,
        }
    }

    pub fn empty() -> Self {
        Snippet {
            message: String::new(),
            source: SourceBuffer::empty(),
            start: 0,
            end: 0,
            line: 0,
            multiline: false,
            padding: 0,
        }
    }

    pub fn set_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn set_source(mut self, source: SourceBuffer) -> Self {
        self.source = source;
        self
    }

    pub fn set_start(mut self, start: usize) -> Self {
        self.start = start;
        self
    }

    pub fn set_end(mut self, end: usize) -> Self {
        self.end = end;
        self
    }

    pub fn set_multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// Gets the line of code that is being reported on.
    /// If this is multi-line, then the line will be the first line of the snippet.
    pub fn get_line(&self) -> usize {
        self.source.get_line_at(self.start).unwrap().line()
    }
}

/// Creates a virtual padded region.
/// This is used to create a region that is padded by the amount of spaces
/// From left to right.
pub struct Padding {
    pub(crate) text: String,
    pub(crate) charset: Charset,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

impl Padding {
    pub fn new(text: String, charset: Charset, size: usize) -> Self {
        Padding {
            text,
            charset,
            left: None,
            right: Some(size),
        }
    }

    pub fn new_even(text: String, charset: Charset, size: usize) -> Self {
        Padding {
            text,
            charset,
            left: Some(size),
            right: Some(size),
        }
    }
}

impl fmt::Display for Padding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut left = String::new();
        let mut right = String::new();

        if let Some(left_size) = self.left {
            for _ in 0..left_size {
                left.push(self.charset.space);
            }
        }

        if let Some(right_size) = self.right {
            for _ in 0..right_size {
                right.push(self.charset.space);
            }
        }

        write!(f, "{}{}{}", left, self.text, right)
    }
}

pub struct Charset {
    pub dash: char,
    pub pipe: char,
    pub space: char,
    pub underline: char,
    pub open: char,
    pub close: char,
    pub rarrow: char,
    pub larrow: char,
}

impl Charset {
    pub fn defaults() -> Self {
        Charset {
            dash: '-',
            pipe: '|',
            space: ' ',
            underline: '~',
            open: '[',
            close: ']',
            rarrow: '>',
            larrow: '<',
        }
    }
}
