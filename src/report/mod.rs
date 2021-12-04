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
    pub source: String,
    pub kind: ReportKind,
}

impl Report {
    pub fn new() -> Self {
        Report {
            code: 0,
            message: String::new(),
            source: String::new(),
            kind: ReportKind::Error,
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

    pub fn set_source(mut self, source: String) -> Self {
        self.source = source;
        self
    }

    pub fn set_kind(mut self, kind: ReportKind) -> Self {
        self.kind = kind;
        self
    }
}
