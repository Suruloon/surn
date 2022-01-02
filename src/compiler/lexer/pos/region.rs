use std::fmt;

use super::Position;

/// This struct holds a "Selection" of text in the cursor!
/// Consider this like a wrapping around a certain context.
///
/// ```
/// rust no_run
/// let new_index = Cursor::select(Region::from(0, 10));
/// ```
#[derive(Debug, Clone)]
pub struct Region {
    pub start: Position,
    pub end: Position,
    label: String,
}

impl Region {
    pub fn new(start: Position, end: Position, name: Option<String>) -> Self {
        Self {
            start,
            end,
            label: match name {
                None => "Region".to_owned(),
                Some(v) => v,
            },
        }
    }

    /// A utility that creates a reagion from to line numbers
    /// with a column of 0.
    ///
    /// **Example:**
    /// Creates a region from line 10 column 0 to line 21 column 0.
    /// ```
    /// rust no_run
    /// let region = Region::from(10, 21);
    /// ```
    pub fn from(line: usize, last: usize) -> Self {
        Self::create(line, 0, last, 0)
    }

    /// A utility that creates a region from `usize`'s.
    ///
    /// **Example:**
    /// Creates a region from line 0, column 0 to line 92 column 304.
    /// ```
    /// rust no_run
    /// let region = Region::create(0, 0, 92, 304);
    /// ```
    pub fn create(start_ln: usize, start_col: usize, end_ln: usize, end_col: usize) -> Self {
        Self::new(
            Position::new(start_ln, start_col),
            Position::new(end_ln, end_col),
            None,
        )
    }

    /// Whether or not a given position is within the current `Region`.
    /// Useful for context and strict tokenization.
    ///
    /// **Example:**
    ///
    /// ```
    /// rust no_run
    /// let region = Region::create(0, 0, 10, 29);
    /// region.includes(&Position::new(3, 4)); // true
    /// ```
    pub fn includes(&self, pos: &Position) -> bool {
        pos.is_leading(&self.start) && self.end.is_leading(&pos)
    }

    /// Gets the name of the current region (if any).
    pub fn get_name(&self) -> String {
        self.label.clone()
    }

    /// Expands the region to the given position.
    pub fn expand_to(&mut self, pos: &Position) -> Self {
        self.end = pos.clone();
        self.clone()
    }

    /// Shrinks the region to the given position.
    pub fn shrink_to(&mut self, pos: &Position) -> Self {
        if pos.is_leading(&self.end) {
            panic!("Given position to shrink to is larger than current position.")
        }
        self.end = pos.clone();
        self.clone()
    }

    // TODO shrink function that shrinks the region from another region.
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Line: {} | Column: {}]",
            self.start.line, self.start.column
        )
    }
}
