pub mod cursor;
pub mod region;

pub use self::region::Region;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Add a position to the current one.
    pub fn add(&mut self, pos: Self) {
        self.line += pos.line;
        self.column += pos.column;
    }

    /// Subtract a position from the current one.
    /// **Example:**
    ///
    /// ```
    /// rust no_run
    /// let pos = Position::new(10, 20);
    /// let pos2 = Position::new(10, 5);
    /// pos.sub(pos2); // Position { line: 0, column: 15 }
    /// ```
    pub fn sub(&mut self, pos: Self) {
        self.line -= pos.line;
        self.column -= pos.column;
    }

    /// Checks whether the current position is"leading" or "ahead"
    /// of the given `Position`. For example:
    /// ```
    /// rust no_run
    /// let pos = Position::new(0, 10);
    /// let new_pos = Position::new(1, 0);
    /// pos::is_leading(&new_pos); // false
    /// new_pos::is_leading(&pos); // true
    /// ```
    pub fn is_leading(&self, pos: &Self) -> bool {
        if self.line > pos.line || self.column > pos.column {
            false
        } else {
            true
        }
    }
}
