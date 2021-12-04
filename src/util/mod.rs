pub mod token_stream;

pub use self::token_stream::TokenStream;

pub trait StreamBuffer {
    type Item;

    /// Peeks the next item in the iterator
    /// Basically a `next` call on the iterator.
    fn peek(&mut self) -> Option<Self::Item>;

    /// Peeks the `n` times in the iterator
    fn peek_inc(&mut self, n: usize) {
        for i in 0..n {
            self.peek();
        }
    }

    /// Attempts to reverse the last peeked item
    /// If the last peeked item was not reversed, it will return `None`
    fn unpeek(&mut self) -> Option<Self::Item>;

    /// Returns whether or not the buffer is empty.
    fn is_eof(&self) -> bool;

    /// Returns the first item in the buffer without removing it.
    fn first(&self) -> Option<Self::Item>;

    /// Returns the second item in the buffer without removing it.
    fn second(&self) -> Option<Self::Item>;

    /// Gets the `nth` item of the buffer without consuming it.
    fn nth(&self, n: usize) -> Option<Self::Item>;

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
