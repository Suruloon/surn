use crate::util::{StreamBuffer, TokenStream};

use super::{
    token::Token,
    tokenizer::tokenize,
};

pub struct Analyzer {
    pub stream: TokenStream,
    // Todo: Properly add errors.
    errors: Vec<String>,
}

impl Analyzer {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            stream: TokenStream::new(tokens),
            errors: Vec::new(),
        }
    }

    pub fn next(&mut self) -> bool {
        // Does a check on identifiers
        if let Err(error) = self.check_identifiers() {
            self.errors.push(error);
        }

        if let Err(error) = self.check_captures() {
            self.errors.push(error);
        }

        self.stream.peek();
        return true;
    }

    /// Does a full analysis of identifiers (if they are present).
    fn check_identifiers(&mut self) -> Result<(), String> {
        if !self.stream.second().is_some() {
            return Ok(());
        }

        if let Some(token) = self.stream.first() {
            let second = self.stream.second().unwrap();
            if token.kind().is_identifier() && second.kind().is_identifier() {
                // Identifiers can NOT be next to eachother!
                self.stream.peek_inc(1);
                return Err(
                    format!("Error! Identifiers can never be next to each-other in this context!\n -> The indentifier \"{}\" at {} is next to identifier \"{}\" at {}", token.value().unwrap(), token.1, second.value().unwrap(), second.1)
                );
            }
        }
        Ok(())
    }

    /// Checks for captures. A capture being anything that is followed by a `(` and ended with a `)`.
    fn check_captures(&mut self) -> Result<(), String> {
        let token = self.stream.first().unwrap();

        if token.kind().is_left_parenthesis() {
            let mut contents = self.stream.clone();
            let mut needs = 0;
            contents.peek();

            while !contents.is_eof() {
                if let Some(ctk) = contents.peek() {
                    if ctk.kind().is_right_parenthesis() {
                        if needs == 0 {
                            return Ok(());
                        } else {
                            needs -= 1;
                        }
                    } else if ctk.kind().is_left_parenthesis() {
                        needs += 1;
                    }
                }
            }
            return Err(format!(
                "Error! Parenthesis at {} is never closed!",
                token.1
            ));
        }

        Ok(())
    }
}

pub fn analyze_source(source: &str) -> Result<(), String> {
    let tokens = tokenize(source);
    let mut analyzer = Analyzer::new(tokens);
    while !analyzer.stream.is_eof() {
        analyzer.next();
    }

    if !analyzer.errors.is_empty() {
        return Err(analyzer.errors.join("\n"));
    }

    Ok(())
}

pub fn analyze(tokens: Vec<Token>) -> Result<(), String> {
    let mut analyzer = Analyzer::new(tokens);
    while !analyzer.stream.is_eof() {
        analyzer.next();
    }

    if !analyzer.errors.is_empty() {
        return Err(analyzer.errors.join("\n"));
    }

    Ok(())
}
