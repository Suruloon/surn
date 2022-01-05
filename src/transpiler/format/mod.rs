pub enum BraceType {
    Allman,
    KandR,
    AllmanMix,
}

pub struct FormatOptions {
    pub tab_size: usize,
    pub indent_size: usize,
    pub new_line: char,
    pub indent_after_type: bool,
    pub snake_case_vars: bool,
    pub class_brace: BraceType,
    pub function_brace: BraceType,
    pub if_brace: BraceType,
    pub else_brace: BraceType,
    pub while_brace: BraceType,
    pub for_brace: BraceType,
    pub match_brace: BraceType,
}

impl FormatOptions {
    pub fn psr_4() -> Self {
        FormatOptions {
            tab_size: 4,
            indent_size: 4,
            new_line: '\n',
            indent_after_type: true,
            snake_case_vars: false,
            class_brace: BraceType::Allman,
            function_brace: BraceType::Allman,
            if_brace: BraceType::Allman,
            else_brace: BraceType::Allman,
            while_brace: BraceType::Allman,
            for_brace: BraceType::Allman,
            match_brace: BraceType::Allman,
        }
    }

    pub fn rust() -> Self {
        FormatOptions {
            tab_size: 4,
            indent_size: 4,
            new_line: '\n',
            indent_after_type: true,
            snake_case_vars: true,
            class_brace: BraceType::KandR,
            function_brace: BraceType::KandR,
            if_brace: BraceType::KandR,
            else_brace: BraceType::KandR,
            while_brace: BraceType::KandR,
            for_brace: BraceType::KandR,
            match_brace: BraceType::KandR,
        }
    }

    pub fn default() -> Self {
        FormatOptions {
            tab_size: 4,
            indent_size: 4,
            new_line: '\n',
            indent_after_type: true,
            snake_case_vars: false,
            class_brace: BraceType::KandR,
            function_brace: BraceType::KandR,
            if_brace: BraceType::KandR,
            else_brace: BraceType::KandR,
            while_brace: BraceType::KandR,
            for_brace: BraceType::KandR,
            match_brace: BraceType::KandR,
        }
    }
}
