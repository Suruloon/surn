use std::{fmt, ops::Range};

use crate::util::source::SourceBuffer;

pub(crate) fn repeat_char(c: char, n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push(c);
    }
    s
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ReportKind {
    Error,
    Warning,
    Notice,
}

impl fmt::Display for ReportKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ReportKind::Error => "Error",
            ReportKind::Warning => "Warning",
            ReportKind::Notice => "Notice",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub struct Report {
    pub code: u64,
    pub message: String,
    pub name: String,
    pub source: SourceBuffer,
    pub snippets: Vec<Snippet>,
    pub kind: ReportKind,
}

impl Report {
    pub fn new() -> Self {
        Report {
            name: String::from("unknown-name.surn"),
            code: 0,
            message: String::new(),
            source: SourceBuffer::empty(),
            snippets: Vec::new(),
            kind: ReportKind::Error,
        }
    }

    pub fn set_code(mut self, code: u64) -> Self {
        self.code = code;
        self
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
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

    pub fn make_snippet(
        mut self,
        range: Range<usize>,
        message: String,
        inline: Option<String>,
    ) -> Self {
        self.snippets.push(
            self.empty_snippet()
                .set_source(self.source.clone())
                .set_range(range)
                .set_message(message)
                .set_inline(inline.unwrap_or("".to_string())),
        );
        self
    }

    pub fn add_snippet(mut self, snippet: Snippet) -> Self {
        self.snippets.push(snippet.set_source(self.source.clone()));
        self
    }

    pub fn print(&self) {
        let main_error = format!("{}! {}", self.kind, self.message);
        let header = format!(
            "{} [{}]",
            repeat_char(Charset::defaults().dash, self.get_width() + 2),
            self.name
        );
        let spacer = format!(
            "{} |",
            repeat_char(Charset::defaults().space, self.get_width())
        );
        let spacer2 = format!(
            "\n{} |\n",
            repeat_char(Charset::defaults().space, self.get_width())
        );
        let snippets = self
            .snippets
            .iter()
            .map(|s| s.get_print())
            .collect::<Vec<String>>();
        // todo: Add error snippets, see error.debug for an example of an error snippet.
        // todo: An error snippet essentially expands the error into possible solutions.
        if self.kind == ReportKind::Error {
            eprint!(
                "{}\n{}\n{}\n{}\n",
                main_error,
                header,
                spacer,
                snippets.join(&spacer2)
            );
        } else {
            print!(
                "{}\n{}\n{}\n{}\n",
                main_error,
                header,
                spacer,
                snippets.join(&spacer2)
            );
        }
    }

    fn get_width(&self) -> usize {
        let mut width = format!("{}", self.source.get_lines().len()).len();
        if width < 3 {
            width = 3;
        }
        width
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
    pub(crate) inline: String,
    source: SourceBuffer,
    multiline: bool,
    range: Range<usize>,
}

impl Snippet {
    pub fn new(source: SourceBuffer, message: String, range: Range<usize>) -> Self {
        Snippet {
            message: message,
            inline: String::new(),
            source,
            range: range,
            multiline: false,
        }
    }

    pub fn empty() -> Self {
        Snippet {
            message: String::new(),
            inline: String::new(),
            source: SourceBuffer::empty(),
            range: (0 as usize)..(1 as usize),
            multiline: false,
        }
    }

    pub fn set_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn set_inline(mut self, inline: String) -> Self {
        self.inline = inline;
        self
    }

    pub fn set_source(mut self, source: SourceBuffer) -> Self {
        self.source = source;
        self
    }

    pub fn set_range(mut self, range: Range<usize>) -> Self {
        self.range = range;
        self
    }

    pub fn set_multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// Gets the line of code that is being reported on.
    /// If this is multi-line, then the line will be the first line of the snippet.
    pub fn get_line(&self) -> usize {
        self.source.get_line_at(self.range.start).unwrap().line()
    }

    pub fn get_print(&self) -> String {
        // generating the padding
        let source_code = format!(
            "{}",
            self.source
                .get_line_at(self.range.start)
                .expect(format!("Could not find line for index at: {}", self.range.start).as_str())
                .trim()
                .source()
        );
        let inlined = format!("{}", self.inline);
        let mut longest = format!("{}", self.source.get_lines().len()).len();
        if longest < 3 {
            longest = 3;
        }
        let line_num =
            SizedPadding::new(format!("{}", self.get_line()), Charset::defaults(), longest);
        let underline = format!(
            "{} |{}",
            repeat_char(Charset::defaults().space, longest),
            format!(
                "{}{} {}",
                repeat_char(
                    Charset::defaults().space,
                    self.source
                        .get_line_at(self.range.start)
                        .unwrap()
                        .spaces_until(self.range.clone())
                ),
                repeat_char(Charset::defaults().underline, self.range.clone().count()),
                inlined
            )
        );

        let message = format!(
            "{} | ---> {}",
            SizedPadding::new("Err".into(), Charset::defaults(), longest),
            self.message
        );
        format!("{} | {}\n{}\n{}", line_num, source_code, underline, message)
    }
}

impl fmt::Display for Snippet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_print())
    }
}

/// Creates a virtual padded region.
/// This is used to create a region that is padded by the amount of spaces
/// From left to right.
pub struct SizedPadding {
    pub(crate) text: String,
    pub(crate) charset: Charset,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

impl SizedPadding {
    pub fn new(text: String, charset: Charset, size: usize) -> Self {
        SizedPadding {
            text,
            charset,
            left: None,
            right: Some(size),
        }
    }

    pub fn new_even(text: String, charset: Charset, size: usize) -> Self {
        SizedPadding {
            text,
            charset,
            left: Some(size),
            right: Some(size),
        }
    }
}

impl fmt::Display for SizedPadding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut left = String::new();
        let mut right = String::new();

        if let Some(left_size) = self.left {
            for _ in 0..left_size {
                left.push(self.charset.space);
            }
        }

        if let Some(right_size) = self.right {
            for _ in 0..(right_size - self.text.len()) {
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
