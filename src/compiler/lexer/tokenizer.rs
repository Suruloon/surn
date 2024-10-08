use super::{
    keyword::{KeyWord, MAX_KEYWORD_LENGTH},
    pos::cursor::{Cursor, END_OF_FILE},
    token::{Token, TokenType},
};

macro_rules! token {
    ($start: expr, $end: expr, $t: expr, $v: expr) => {
        Some(Token($t, $start..$end, $v))
    };
    ($start: expr, $end: expr, $t: expr) => {
        Some(Token($t, $start..$end, None))
    };
}

impl Cursor<'_> {
    fn eat(&mut self) -> Option<Token> {
        let start_pos = self.get_pos();

        if let Some(spaces) = self.eat_whitespace() {
            return token!(
                start_pos,
                self.get_pos(),
                TokenType::Whitespace,
                Some(spaces)
            );
        }

        if let Some(comment) = self.eat_comment() {
            return token!(start_pos, self.get_pos(), TokenType::Comment, Some(comment));
        }

        if let Some(operator) = self.eat_operator() {
            return token!(
                start_pos,
                self.get_pos(),
                TokenType::Operator,
                Some(operator)
            );
        }

        if let Some(keyword) = self.eat_keyword() {
            return token!(
                start_pos,
                self.get_pos(),
                TokenType::KeyWord(KeyWord::from_string(&keyword).unwrap()),
                None
            );
        }

        if let Some(boolean) = self.eat_boolean() {
            return token!(start_pos, self.get_pos(), TokenType::Boolean, Some(boolean));
        }

        if let Some(identifier) = self.eat_identifier() {
            return token!(
                start_pos,
                self.get_pos(),
                TokenType::Identifier,
                Some(identifier)
            );
        }

        if let Some(number) = self.eat_number() {
            return token!(start_pos, self.get_pos(), TokenType::Number, Some(number));
        }

        if let Some(string) = self.eat_string() {
            self.peek(); // what?
            return token!(
                start_pos,
                self.get_pos(),
                TokenType::StringLiteral,
                Some(string)
            );
        }

        if let Some(token_type) = self.eat_value_reserved() {
            return token!(start_pos, self.get_pos(), token_type.0, Some(token_type.1));
        }

        if let Some(token_type) = self.eat_reserved() {
            // Peek if a reserved character is found
            self.peek();
            return token!(start_pos, self.get_pos(), token_type);
        }

        self.peek();
        return None;
    }

    fn eat_comment(&mut self) -> Option<String> {
        return match self.first() {
            '/' => {
                // check the next character
                if self.second() == '/' {
                    Some(self.eat_while(|c| c != '\n'))
                } else if self.second() == '*' {
                    // eat the comment
                    let comment = self.eat_while_cursor(|cursor, c| {
                        if c == '*' {
                            if cursor.first() == '/' {
                                cursor.eat();
                                return false;
                            } else {
                                return true;
                            }
                        } else {
                            return true;
                        }
                    });
                    Some(comment)
                } else {
                    None
                }
            }
            _ => None,
        };
    }

    /// This may be misleading,
    /// because it eats ALL whitespace until a char is not whitespace
    fn eat_whitespace(&mut self) -> Option<String> {
        let segment = self.eat_while(|c| c.is_whitespace());
        return if segment.is_empty() {
            None
        } else {
            Some(segment)
        };
    }

    fn eat_identifier(&mut self) -> Option<String> {
        match self.first() {
            // 'A'..='z' can't be used here as it includes a plethora of reserved characters that are used elsewhere
            '_' | 'a'..='z' | 'A'..='Z' => Some(
                self.eat_while(|c: char| !c.is_whitespace() && (c.is_alphanumeric() || c == '_')),
            ),
            _ => None,
        }
    }

    fn eat_number(&mut self) -> Option<String> {
        match self.first() {
            // there is an issue with leading floats where they are parsed as accessors right now.
            // we should leave this to the parser.
            '0'..='9' => Some(self.eat_while(|c: char| c.is_digit(10) || c == '.')),
            _ => None,
        }
    }

    /// Eats a keyword but does not parse it.
    fn eat_keyword(&mut self) -> Option<String> {
        let mut segment = String::new();
        for i in 0..MAX_KEYWORD_LENGTH {
            let next_char = self.nth_char(i);
            if next_char == END_OF_FILE {
                return None;
            }
            segment.push(next_char);

            if let Some(keyword) = KeyWord::from_string(&segment) {
                if self.nth_char(i + 1).is_whitespace() {
                    self.peek_inc(i);
                    return Some(keyword.to_string());
                } else {
                    return None;
                }
            }
        }

        return None;
    }

    fn eat_operator(&mut self) -> Option<String> {
        match self.first() {
            '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' | '&' | '|' | '^' | '~' => {
                self.peek();
                Some(self.get_prev().to_string())
            }
            'o' => {
                if self.nth_char(1) == 'r' {
                    self.peek_inc(2);
                    Some("or".to_string())
                } else {
                    None
                }
            }
            'a' => {
                if self.nth_char(1) == 'n' && self.nth_char(2) == 'd' {
                    self.peek_inc(3);
                    return Some("and".to_string());
                } else {
                    return None;
                }
            }
            _ => None,
        }
    }

    fn eat_boolean(&mut self) -> Option<String> {
        // TODO: it may not be the best practice to use this vector
        for value in ["true", "false"].iter() {
            let mut segment = String::new();
            for i in 0..value.len() {
                let next_char = self.nth_char(i);
                if next_char == END_OF_FILE {
                    return None;
                }

                // Next character doesn't match current value
                // Move to next value if one exists
                if next_char != value.chars().nth(i).unwrap() {
                    continue;
                }
                segment.push(next_char);

                if segment == value.to_string() {
                    self.peek_inc(i);
                    return Some(segment);
                }
            }
        }
        return None;
    }

    fn eat_string(&mut self) -> Option<String> {
        if self.first() != '"' && self.first() != '\'' && self.first() != '`' {
            return None;
        } else {
            let first = self.peek().unwrap();
            return Some(self.eat_while(|c| c != first));
        }
    }

    fn eat_value_reserved(&mut self) -> Option<(TokenType, String)> {
        match self.first() {
            ':' => {
                if self.second() == ':' {
                    self.peek_inc(1);
                    return Some((TokenType::Accessor, "::".to_string()));
                } else {
                    self.peek();
                    return Some((TokenType::Colon, ":".to_string()));
                }
            }
            '.' => {
                if self.second() == '.' {
                    self.peek_inc(2);
                    return Some((TokenType::Range, "..".to_string()));
                } else {
                    self.peek();
                    return Some((TokenType::Accessor, ".".to_string()));
                }
            }
            _ => None,
        }
    }

    fn eat_reserved(&mut self) -> Option<TokenType> {
        match self.first() {
            '[' => Some(TokenType::LeftBracket),
            ']' => Some(TokenType::RightBracket),
            '(' => Some(TokenType::LeftParenthesis),
            ')' => Some(TokenType::RightParenthesis),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
            ';' => Some(TokenType::StatementEnd),
            ',' => Some(TokenType::Comma),
            '\\' => Some(TokenType::Backslash),
            _ => None,
        }
    }
}

pub fn tokenize<'a>(input: &'a str) -> Vec<Token> {
    let mut cursor = Cursor::new(input);
    let mut tokens: Vec<Token> = Vec::new();

    while !cursor.is_eof() {
        let token = cursor.eat();
        if let Some(token) = token {
            tokens.push(token);
        }
    }

    return tokens;
}
