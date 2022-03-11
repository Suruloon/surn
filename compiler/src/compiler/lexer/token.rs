use crate::compiler::lexer::keyword::KeyWord;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// Any token that signifies a variable is to be created.
    ///
    /// For example:
    /// - `const`
    Constant,
    /// Any token that signifies a variable is to be created.
    /// For example:
    /// - `var`
    Variable,
    // The operator token that is used to assign a value to a variable.
    // For example:
    // - `:`
    Colon,
    /// Any characters that follow `#` or `//`.
    ///
    /// For example:
    /// - `# This is a comment`
    /// - `// This is a comment`
    Comment,
    /// Any word that is considered a "keyword" otherwise reserved by the compiler.
    /// For example:
    /// - `if`
    /// - `else`
    /// - `other`
    KeyWord(KeyWord),
    /// A phrase is anything that is consumed and rendered "unknown"
    /// but is a phrase that is not a keyword.
    ///
    /// For example:
    /// - `bat`
    /// - `dog`
    Identifier,
    /// A number is a sequence of digits that is not a keyword.
    /// For example:
    /// - `123`
    /// - `0.123`
    Number,
    /// A string is a sequence of characters that is not a keyword.
    ///
    /// For example:
    /// - `"Hello World"`
    /// - `'Goodbye World'`
    /// - `surn is an awesome transpiler!'`
    StringLiteral,
    /// An operator is a character that operates on arguments and produces a value.
    ///
    /// For example:
    /// - `+`
    /// - `-`
    /// - `*`
    /// - `/`
    /// - `%`
    /// - `^`
    /// - `>`
    /// - `<`
    /// - `=`
    /// - `!`
    /// - `&`
    /// - `|`
    /// - `and`
    /// - `or`
    /// - `not`
    Operator,
    /// An accessor is a character that accesses a value.
    /// For example:
    /// - `.`
    /// - `::`
    /// - `->`
    Accessor,
    /// A range token.
    /// For example:
    /// - `..`
    Range,
    /// The keywords `true` and `false` are used to represent boolean values.
    /// For example:
    /// - `var test: bool = true;`
    /// - `var apple: bool = false;`
    Boolean,
    /// A whitespace character is a character that is not a keyword, number, or string.
    Whitespace,
    /// The semi-colon character that signals the end of a statement.
    ///
    /// For example:
    /// - `;`
    StatementEnd,
    /// The `\n` sequence of characters that signals the end of a line.
    LineBreak,
    /// The `[` character that can signal a return type, an array, etc.
    LeftBracket,
    /// The `]` character that can signal the end of a return type, an array, etc.
    RightBracket,
    /// The `(` character that can signal the start of a function call.
    LeftParenthesis,
    /// The `)` character that can signal the end of a function call.
    RightParenthesis,
    /// The `{` character that can signal the start of a block.
    LeftBrace,
    /// The `}` character that can signal the end of a block.
    RightBrace,
    /// The `,` character that can signal the end of a parameter.
    Comma,
    /// The `\` character that can signal the start of a string literal.
    Backslash,
}

impl TokenType {
    pub fn is_colon(&self) -> bool {
        match self {
            TokenType::Colon => true,
            _ => false,
        }
    }

    pub fn is_keyword(&self) -> bool {
        match self {
            TokenType::KeyWord(_) => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool {
        match self {
            TokenType::Operator => true,
            _ => false,
        }
    }

    pub fn is_statement_end(&self) -> bool {
        match self {
            TokenType::StatementEnd => true,
            _ => false,
        }
    }

    pub fn is_line_break(&self) -> bool {
        match self {
            TokenType::LineBreak => true,
            _ => false,
        }
    }

    pub fn is_comment(&self) -> bool {
        match self {
            TokenType::Comment => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            TokenType::StringLiteral => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            TokenType::Number => true,
            _ => false,
        }
    }

    pub fn is_identifier(&self) -> bool {
        match self {
            TokenType::Identifier => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            TokenType::Boolean => true,
            _ => false,
        }
    }

    pub fn is_variable(&self) -> bool {
        match self {
            TokenType::Variable => true,
            _ => false,
        }
    }

    pub fn is_assignment(&self) -> bool {
        match self {
            TokenType::Colon => true,
            _ => false,
        }
    }

    pub fn is_left_bracket(&self) -> bool {
        match self {
            TokenType::LeftBracket => true,
            _ => false,
        }
    }

    pub fn is_right_bracket(&self) -> bool {
        match self {
            TokenType::RightBracket => true,
            _ => false,
        }
    }

    pub fn is_left_parenthesis(&self) -> bool {
        match self {
            TokenType::LeftParenthesis => true,
            _ => false,
        }
    }

    pub fn is_right_parenthesis(&self) -> bool {
        match self {
            TokenType::RightParenthesis => true,
            _ => false,
        }
    }

    pub fn is_left_brace(&self) -> bool {
        match self {
            TokenType::LeftBrace => true,
            _ => false,
        }
    }

    pub fn is_whitespace(&self) -> bool {
        match self {
            TokenType::Whitespace => true,
            _ => false,
        }
    }

    pub fn is_right_brace(&self) -> bool {
        match self {
            TokenType::RightBrace => true,
            _ => false,
        }
    }

    pub fn is_comma(&self) -> bool {
        match self {
            TokenType::Comma => true,
            _ => false,
        }
    }

    pub fn is_constant(&self) -> bool {
        match self {
            TokenType::Constant => true,
            _ => false,
        }
    }

    pub fn is_accessor(&self) -> bool {
        match self {
            TokenType::Accessor => true,
            _ => false,
        }
    }

    pub fn is_range(&self) -> bool {
        match self {
            TokenType::Range => true,
            _ => false,
        }
    }

    pub fn is_backslash(&self) -> bool {
        match self {
            TokenType::Backslash => true,
            _ => false,
        }
    }

    /// This will panic if the token type is not a keyword.
    pub fn as_keyword(&self) -> KeyWord {
        match self {
            TokenType::KeyWord(keyword) => keyword.clone(),
            _ => panic!("Token type is not a keyword but a keyword was expected."),
        }
    }
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::Variable => "Variable".to_string(),
            TokenType::Constant => "Constant".to_string(),
            TokenType::Colon => "Colon".to_string(),
            TokenType::Comment => "Comment".to_string(),
            TokenType::KeyWord(_) => "KeyWord".to_string(),
            TokenType::Identifier => "Identifier".to_string(),
            TokenType::Number => "Number".to_string(),
            TokenType::StringLiteral => "String".to_string(),
            TokenType::Operator => "Operator".to_string(),
            TokenType::StatementEnd => "Statement End".to_string(),
            TokenType::LineBreak => "EndOfLine".to_string(),
            TokenType::Boolean => "Boolean".to_string(),
            TokenType::LeftBracket => "Opening Delimiter".to_string(),
            TokenType::RightBracket => "Closing Delimiter".to_string(),
            TokenType::LeftParenthesis => "Opening Delimiter".to_string(),
            TokenType::RightParenthesis => "Closing Delimiter".to_string(),
            TokenType::LeftBrace => "Opening Delimiter".to_string(),
            TokenType::RightBrace => "Closing Delimiter".to_string(),
            TokenType::Comma => "Comma".to_string(),
            TokenType::Whitespace => "Whitespace".to_string(),
            TokenType::Accessor => "Accessor".to_string(),
            TokenType::Range => "Range".to_string(),
            TokenType::Backslash => "Backslash".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token(pub TokenType, pub Range<usize>, pub Option<String>);

impl Token {
    pub fn kind(&self) -> TokenType {
        self.0.clone()
    }

    pub fn value(&self) -> Option<String> {
        self.2.clone()
    }

    pub fn range(&self) -> Range<usize> {
        self.1.clone()
    }
}
