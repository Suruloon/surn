use std::collections::VecDeque;

use crate::compiler::lexer::token::Token;

use super::StreamBuffer;

#[derive(Debug, Clone)]
struct TokenIterator {
    pub(super) buffer: VecDeque<Token>,
}

impl TokenIterator {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            buffer: VecDeque::from(tokens),
        }
    }
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }
}

/// A useful utility for handling of tokens.
#[derive(Debug, Clone)]
pub struct TokenStream {
    /// A constantly resized vector of tokens
    /// this is internally an iterator over tokens.
    buffer: TokenIterator,
    /// The initial length of the buffer before iterating
    /// over it.
    initial_length: usize,
    prev: Option<Token>,
}

impl TokenStream {
    /// Creates a new token stream with the given initial length.
    pub fn new(tokens: Vec<Token>) -> TokenStream {
        TokenStream {
            buffer: TokenIterator::new(tokens.clone()),
            initial_length: tokens.len(),
            prev: None,
        }
    }
}

impl StreamBuffer for TokenStream {
    type Item = Token;

    /// Peeks the next item in the iterator
    /// Basically a `next` call on the iterator.
    fn peek(&mut self) -> Option<Self::Item> {
        let next = self.buffer.next();
        self.prev = next.clone();
        next
    }

    /// Attempts to reverse the last peeked item
    /// If the last peeked item was not reversed, it will return `None`
    fn unpeek(&mut self) -> Option<Self::Item> {
        // TODO: Implement this properly
        None
    }

    /// Returns the last peeked item
    /// If the last peeked item was not reversed, it will return `None`
    fn prev(&self) -> Option<Self::Item> {
        self.prev.clone()
    }

    /// Returns whether or not the buffer is empty.
    fn is_eof(&self) -> bool {
        self.buffer.buffer.is_empty()
    }

    /// Returns the first item in the buffer without removing it.
    fn first(&self) -> Option<Self::Item> {
        self.nth(0)
    }

    /// Returns the second item in the buffer without removing it.
    fn second(&self) -> Option<Self::Item> {
        self.nth(1)
    }

    /// Gets the `nth` item of the buffer without consuming it.
    fn nth(&self, n: usize) -> Option<Self::Item> {
        let mut soft_copy = self.buffer.clone();
        soft_copy.nth(n)
    }

    /// Returns a copy of the buffer without consuming it.
    fn items(&self) -> Vec<Self::Item> {
        self.buffer.buffer.clone().into_iter().collect()
    }

    /// Returns the amount of items in the buffer have been consumed.
    fn eaten(&self) -> usize {
        self.initial_length - self.buffer.buffer.len()
    }
}
