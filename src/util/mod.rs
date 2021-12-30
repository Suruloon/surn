pub mod source;
pub mod token_stream;

pub use self::token_stream::TokenStream;

pub trait StreamBuffer {
    type Item;

    /// Peeks the next item in the iterator
    /// Basically a `next` call on the iterator.
    fn peek(&mut self) -> Option<Self::Item>;

    fn peek_or(&mut self, v: Self::Item) -> Self::Item {
        self.peek().unwrap_or(v)
    }

    /// Peeks the interator until the given predicate returns true
    /// This uses `first_if` to determine if the predicate is true
    /// If it is, the iterator is advanced to the next item
    fn peek_until(&mut self, f: impl Fn(&Self::Item) -> bool) -> Option<Self::Item> {
        loop {
            if self.is_eof() {
                return None;
            }
            match self.first_if(|x| !f(x)) {
                Some(_) => {
                    self.peek();
                }
                None => return Some(self.first().unwrap()),
            };
        }
    }

    /// Peeks the next item in the iterator if the predicate is true
    /// Otherwise, returns None and **Does not consume** the iterator.
    fn peek_if(&mut self, f: impl FnOnce(&Self::Item) -> bool) -> Option<Self::Item> {
        if let Some(tk) = self.first_if(f) {
            self.peek();
            Some(tk)
        } else {
            None
        }
    }

    /// Peeks the `n` times in the iterator
    fn peek_inc(&mut self, n: usize) {
        for _ in 0..n {
            self.peek();
        }
    }

    /// Attempts to reverse the last peeked item
    /// If the last peeked item was not reversed, it will return `None`
    fn unpeek(&mut self) -> Option<Self::Item>;

    /// Attempts to reverse the last peeked item
    fn prev(&self) -> Option<Self::Item>;

    /// Returns whether or not the buffer is empty.
    fn is_eof(&self) -> bool;

    /// Returns the first item in the buffer without removing it.
    fn first(&self) -> Option<Self::Item>;

    /// Returns the first item in the buffer without removing it or a default value.
    fn first_or(&self, v: Self::Item) -> Self::Item {
        self.first().unwrap_or(v)
    }

    /// Returns the first item in the buffer without removing it only if the predicate is true.
    fn first_if(&self, f: impl FnOnce(&Self::Item) -> bool) -> Option<Self::Item> {
        self.first().filter(|i| f(i))
    }

    /// Returns the second item in the buffer without removing it.
    fn second(&self) -> Option<Self::Item>;

    fn second_or(&self, v: Self::Item) -> Self::Item {
        self.second().unwrap_or(v)
    }

    /// Returns the second item in the buffer without removing it only if the predicate is true.
    fn second_if(&self, f: impl FnOnce(&Self::Item) -> bool) -> Option<Self::Item> {
        self.second().filter(|i| f(i))
    }

    /// Gets the `nth` item of the buffer without consuming it.
    fn nth(&self, n: usize) -> Option<Self::Item>;

    /// Gets the `nth` item of the buffer without consuming it or a default value.
    fn nth_or(&self, n: usize, v: Self::Item) -> Self::Item {
        self.nth(n).unwrap_or(v)
    }

    /// Gets the `nth` item of the buffer without consuming it only if the predicate is true.
    /// If the predicate is false, it will return `None`.
    fn nth_if(&self, n: usize, f: impl FnOnce(&Self::Item) -> bool) -> Option<Self::Item> {
        self.nth(n).filter(|i| f(i))
    }

    /// Finds the next item in the buffer that matches the predicate.
    /// only after the first predicate is true.
    ///
    /// Returns a tuple that matches the following:
    /// 1. The index of the item that matched the predicate starting from 0
    /// 2. The item that matched the predicate
    /// Example:
    /// ```rust
    /// let mut iter = TokenStream::new("a            z");
    /// iter.peek();
    /// assert_eq!(
    ///     iter.find_after(
    ///         |x| x.is_alphabetic(),
    ///         |x| x.is_whitespace()
    ///     ),
    ///     Some('z')
    /// ); // (14, 'z')
    /// ```
    fn find_after(
        &mut self,
        find: impl Fn(&Self::Item) -> bool,
        after: impl Fn(&Self::Item) -> bool,
    ) -> Option<(usize, Self::Item)> {
        let mut i = 0;
        loop {
            if let Some(_) = self.nth_if(i, |t| after(t)) {
                i += 1;
            } else if let Some(tk) = self.nth_if(i, find) {
                return Some((i, tk));
            } else {
                return None;
            }
        }
    }


    /// Similar to `find_after`, but returns the `nth` item of the buffer without consuming it.
    /// If the `nth` item is not found, it will return `None`.
    fn find_after_nth(
        &mut self,
        nth: usize,
        find: impl Fn(&Self::Item) -> bool,
        after: impl Fn(&Self::Item) -> bool,
    ) -> Option<(usize, Self::Item)> {
        let mut i = nth;
        loop {
            if let Some(_) = self.nth_if(i, |t| after(t)) {
                i += 1;
            } else if let Some(tk) = self.nth_if(i, find) {
                return Some((i, tk));
            } else {
                return None;
            }
        }
    }

    /// Returns a copy of the buffer without consuming it.
    fn items(&self) -> Vec<Self::Item>;

    /// Returns the amount of items in the buffer have been consumed.
    fn eaten(&self) -> usize;

    /// Consumes `o(n)` items from the buffer and returns them until
    /// the predicate returns `true`.
    fn eat_while<F>(&mut self, mut predicate: F) -> Vec<Self::Item>
    where
        F: FnMut(Self::Item) -> bool,
    {
        let mut items: Vec<Self::Item> = Vec::new();
        while !self.is_eof() {
            // get the next item without consuming it
            if let Some(next) = self.first() {
                // if the predicate returns true, push the item to the vector
                // an proceed, otherwise break out of the loop
                if predicate(next) {
                    items.push(self.peek().unwrap());
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        return items;
    }
}
