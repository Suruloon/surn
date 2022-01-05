use std::ops::Range;

use crate::util::TokenStream;

use self::{
    context::{ContextStore, SourceOrigin},
    generator::AstGenerator,
};

use super::{
    ast::AstBody,
    lexer::{analysis::analyze, token::Token, tokenizer::tokenize},
    CompilerOptions,
};

pub mod context;
pub mod generator;

#[derive(Debug, Clone)]
pub struct ParserError {
    pub reason: String,
    pub message: String,
    pub extra: Option<String>,
    pub location: Range<usize>,
    pub ast: AstBody,
}

impl ParserError {
    pub fn new(
        reason: String,
        message: String,
        location: Range<usize>,
        ast: AstBody,
        extra: Option<String>,
    ) -> Self {
        ParserError {
            reason,
            message,
            extra,
            location,
            ast,
        }
    }

    pub fn set_inline(&mut self, inline: String) {
        self.extra = Some(inline);
    }
}

/// The parser struct.
/// This contains the context of the AST as well as information
/// regarding errors and warnings with the source code.
pub struct Parser {
    options: CompilerOptions,
    contexts: ContextStore,
}

impl Parser {
    pub fn new(options: CompilerOptions) -> Self {
        Parser {
            options,
            contexts: ContextStore::new(),
        }
    }

    pub fn parse_script(&mut self, name: String, source: String) -> Result<AstBody, ParserError> {
        // create a source origin for the script
        let source_origin = SourceOrigin::new_virtual(name, source.clone());
        // because we're going to be parsing a single script, we can use a new astgenerator.
        let mut ast_generator = AstGenerator::new(source_origin, self.contexts.next_context_id());
        // add the generators context to our parser.
        self.contexts.add_context(&mut ast_generator.context);

        // lets tokenize the source code.
        let tokens = tokenize(source.as_str());

        // do our options with compiler options
        self.do_options(&tokens);

        // time to parse.
        let ast = ast_generator.begin_parse(TokenStream::new(tokens))?; // parse the tokens.

        // remove the context from the parser because it's useless to the parser.
        self.contexts.remove_context(ast_generator.context.origin);
        return Ok(ast);
    }

    pub(crate) fn do_options(&self, tokens: &Vec<Token>) {
        if self.options.semantic_checks == true {
            // do semantic checks
            analyze(tokens.clone()).expect("Error running checks.");
        }
    }
}
