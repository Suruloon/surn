use super::Position;
use std::str::Chars;

pub const END_OF_FILE: char = '\0';

/// A struct that handles a stream of chars
pub struct Cursor<'a> {
    ilen: usize,
    chars: Chars<'a>,
    prev: char,
    pos: Position,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            ilen: input.len(),
            chars: input.chars(),
            prev: END_OF_FILE,
            pos: Position::new(1, 0),
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => {
                self.prev = c;

                if is_line_ending(c) && self.is_eof() {
                    return None;
                }

                if is_line_ending(c) {
                    self.pos.line += 1;
                    self.pos.column = 0;
                } else {
                    self.pos.column += 1;
                }

                Some(c)
            }
            None => None,
        }
    }

    pub fn unpeek(&mut self) -> char {
        self.chars.nth(self.eaten() - 1).unwrap_or(END_OF_FILE)
    }

    /// Is End of file?
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    // Grabs the next char without consuming it.
    pub fn first(&self) -> char {
        self.nth_char(0)
    }

    // Grabs the second char without consuming it.
    pub fn second(&self) -> char {
        self.nth_char(1)
    }

    /// Returns the `nth_char` releative to the current cursor pos
    /// If the position given doesn't exist, `END_OF_FILE` is returned.
    pub fn nth_char(&self, amt: usize) -> char {
        self.chars().nth(amt).unwrap_or(END_OF_FILE)
    }

    /// Copies the current chars in the cursor.
    pub fn chars(&self) -> Chars<'a> {
        self.chars.clone()
    }

    pub fn get_pos(&self) -> Position {
        self.pos.clone()
    }

    pub fn get_prev(&self) -> char {
        self.prev.clone()
    }

    pub fn peek_get_pos(&mut self) -> Position {
        self.peek();
        self.pos.clone()
    }

    pub fn ipeek_get_pos(&mut self, x: usize) -> Position {
        self.peek_inc(x);
        self.pos.clone()
    }

    /// Increments the current buffer with the given one.
    /// Peeks `x` times
    pub fn peek_inc(&mut self, x: usize) {
        let mut i = 0;
        while !self.is_eof() && i <= x {
            self.peek();
            i += 1;
        }
    }

    /// Shows how many chars have been consumed by the cursor.
    pub fn eaten(&self) -> usize {
        self.ilen - self.chars.as_str().len()
    }

    pub fn eat_while(&mut self, mut pred: impl FnMut(char) -> bool) -> String {
        let mut segment = String::new();
        while !self.is_eof() && pred(self.first()) == true {
            segment.push(self.peek().unwrap_or(END_OF_FILE));
        }
        segment
    }

    pub fn eat_while_cursor(
        &mut self,
        mut pred: impl FnMut(&mut Cursor<'a>, char) -> bool,
    ) -> String {
        let mut segment = String::new();
        while !self.is_eof() && pred(self, self.first()) == true {
            segment.push(self.peek().unwrap_or(END_OF_FILE));
        }
        segment
    }
}

fn is_line_ending(c: char) -> bool {
    c == '\n'
}
