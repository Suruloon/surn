use std::process;

use crate::{
    ast::{
        AstBody, Class, ClassProperty, Expression, Function, FunctionInput, Statement, Variable,
        Visibility,
    },
    lexer::{
        analysis::analyze,
        keyword::KeyWord,
        token::{Token, TokenType},
        tokenizer::tokenize,
    },
    report::Report,
    types::{Type, TypeKind, TypeRef},
    util::{source::SourceBuffer, StreamBuffer, TokenStream},
    CompilerOptions,
};

use self::context::{Context, ContextFlag, ContextStore, SourceOrigin};

pub mod context;

macro_rules! create_report {
    ($ctx: expr, $location: expr, $message: expr) => {
        Report::new()
            .set_source(SourceBuffer::new(
                $ctx.source.clone().get_contents().unwrap(),
            ))
            .set_name($ctx.source.clone().name)
            .set_message("Occurred while parsing".to_string())
            .make_snippet($location, $message, None)
            .print();
        process::exit(1);
    };
    ($ctx: expr, $location: expr, $message: expr, $inline: expr) => {
        Report::new()
            .set_source(SourceBuffer::new(
                $ctx.source.clone().get_contents().unwrap(),
            ))
            .set_name($ctx.source.clone().name)
            .set_message("Occurred while parsing".to_string())
            .make_snippet($location, $message, Some($inline))
            .print();
        process::exit(1);
    };
}

/// Returns the current token stack
pub(crate) struct TokenStack {
    /// A vector of tokens with their location in the token stream
    pub tokens: Vec<(usize, Token)>,
}

impl TokenStack {
    pub fn new() -> Self {
        TokenStack { tokens: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
    }

    pub fn push(&mut self, idx: usize, token: Token) {
        self.tokens.push((idx, token));
    }

    pub fn first(&self) -> Option<Token> {
        if let Some((_, token)) = self.tokens.get(0) {
            Some(token.clone())
        } else {
            None
        }
    }

    pub fn nth(&self, n: usize) -> Option<Token> {
        if let Some((_, token)) = self.tokens.get(n) {
            Some(token.clone())
        } else {
            None
        }
    }

    pub fn nth_or(&self, n: usize, token: Token) -> Token {
        if let Some((_, token)) = self.tokens.get(n) {
            token.clone()
        } else {
            token
        }
    }

    pub fn nth_unwrap(&self, n: usize) -> Token {
        if let Some((_, token)) = self.tokens.get(n) {
            token.clone()
        } else {
            panic!("TokenStack::nth_unwrap: index out of bounds");
        }
    }
}

pub struct AstGenerator {
    pub(crate) body: AstBody,
    pub(crate) tokens: TokenStream,
    pub(crate) context: Context,
    stack: TokenStack,
}

/// Parses the given token stream into an AST.
/// Returns a Result containing the AST.
/// AST is **not** optimized during this stage, however it is validated.
impl AstGenerator {
    pub fn new(source: SourceOrigin, id: u64) -> Self {
        AstGenerator {
            body: AstBody::new(),
            tokens: TokenStream::new(Vec::new()),
            context: Context::new(source, id),
            stack: TokenStack::new(),
        }
    }

    pub fn begin_parse(&mut self, tokens: TokenStream) -> AstBody {
        self.tokens = tokens;

        while !self.tokens.is_eof() {
            self.skip_whitespace();
            self.parse();
        }

        return self.body.clone();
    }

    fn parse(&mut self) {
        // attempt to parse a statement
        if let Some(stmt) = self.parse_statement() {
            self.body.push_statement(stmt);
            return;
        }

        if let Some(expr) = self.parse_expression() {
            self.body.push_expression(expr);
            return;
        }

        // we don't know what this is!
        // the only body we can have is a statement or an expression
        create_report!(
            self.context,
            self.tokens.first().unwrap().range(),
            "Unable to proceed parsing. This token was unexpected at this time.".to_string(),
            format!(
                "Unexpected token: {}",
                self.tokens.peek().unwrap().kind().to_string()
            )
        );
    }

    /// A statement can be a variable declaration, function declaration, class declaration, etc.
    fn parse_statement(&mut self) -> Option<Statement> {
        // because we're making this top level, we can parse all statements in the global context.

        // try to parse a mutable variable.
        if let Some((var, constant)) = self.parse_variable() {
            if constant {
                return Some(Statement::Const(var));
            } else {
                return Some(Statement::Var(var));
            }
        }

        return None;
    }

    /// Parses a variable declaration (if plausible)
    ///
    /// For example:
    /// - `var x = 5`
    /// - `const x = 5`
    fn parse_variable(&mut self) -> Option<(Variable, bool)> {
        let decl_keyword = self.tokens.peek_if(|t| {
            if t.kind().is_keyword() {
                return (t.kind().as_keyword() == KeyWord::Const)
                    || (t.kind().as_keyword() == KeyWord::Var);
            } else {
                return false;
            }
        });

        if let Some(keyword) = decl_keyword {
            let is_constant = keyword.kind().as_keyword() == KeyWord::Const;
            self.skip_whitespace_err("A variable name was expected but none was found.");

            // check if the next token is an indentifier
            if let Some(identifier) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                let mut type_node: Type;

                // token is an identifier!
                // we need to check if a colon follows, if so, we need to parse a type, otherwise we can skip
                // the type checking and just parse the variable
                if let Some(_) = self.tokens.peek_if(|t| t.kind().is_colon()) {
                    // now parse a type statement.
                    if let Some(type_smt) = self.parse_type_kind() {
                        type_node = type_smt;
                    } else {
                        create_report!(
                            self.context,
                            self.tokens.first().unwrap().range(),
                            "Expected type statement to follow a variable declaration with a colon.".to_string(),
                            "A type statement is expected here.".to_string()
                        );
                    }
                } else {
                    type_node = Type::uninit();
                }

                // we now need an assignment operator
                self.skip_whitespace_err("An operator was expected but none was found.");

                // check for an "equals" operator
                if let Some(_) = self
                    .tokens
                    .peek_if(|t| t.kind().is_operator() && (t.value().unwrap() == "=".to_string()))
                {
                    // we have an equals operator!
                    // we need to parse an expression
                    self.skip_whitespace_err("An expression was expected but none was found.");
                    if let Some(expr) = self.parse_expression() {
                        // we have an expression!
                        // we need to parse a semicolon
                        self.skip_whitespace_err("A semicolon was expected but none was found.");
                        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                            return Some((
                                Variable::new(
                                    identifier.value().unwrap(),
                                    TypeRef::empty(),
                                    Visibility::Private,
                                    Some(expr),
                                ),
                                is_constant,
                            ));
                        } else {
                            create_report!(
                                self.context,
                                self.tokens.first().unwrap().range(),
                                "Expected a semicolon to follow a variable declaration."
                                    .to_string(),
                                "A semicolon is expected here.".to_string()
                            );
                        }
                    } else {
                        create_report!(
                            self.context,
                            self.tokens.first().unwrap().range(),
                            "Expected an expression to follow a variable declaration.".to_string(),
                            "An expression is expected here.".to_string()
                        );
                    }
                } else {
                    // variables **can** be uninitialized
                    // we need to check if the next token is an end of statement
                    if let Some(end_of_statement) =
                        self.tokens.peek_if(|t| t.kind().is_statement_end())
                    {
                        // we have an end of statement!
                        // we can return a variable declaration
                        return Some((
                            Variable::new(
                                identifier.value().unwrap(),
                                TypeRef::empty(),
                                Visibility::Private,
                                None,
                            ),
                            is_constant,
                        ));
                    } else {
                        // we don't have an end of statement!
                        // we need to report an error
                        create_report!(
                            self.context,
                            self.tokens.first().unwrap().range(),
                            "Expected an end of statement to follow an uninitialized declaration."
                                .to_string(),
                            "A semi-colon is expected here.".to_string()
                        );
                    }
                }
            } else {
                create_report!(
                    self.context,
                    self.tokens.first().unwrap().range(),
                    "A name must follow a variable declaration".to_string(),
                    format!(
                        "Unexpected token: \"{}\"",
                        self.tokens.first().unwrap().kind().to_string()
                    )
                );
            }
        } else {
            return None;
        }
    }

    /// Parses a type kind.
    /// For example:
    /// - `int`
    /// - `string`
    /// - `bool`
    fn parse_type_kind(&mut self) -> Option<Type> {
        None
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        // we need to parse an expression
        None
    }

    fn skip_whitespace_err(&mut self, err: &'static str) {
        let start = self.tokens.first().unwrap().range().start;
        match self.tokens.peek_until(|t| !t.kind().is_whitespace()) {
            None => {
                create_report!(
                    self.context,
                    start..self.context.source.get_contents().unwrap().len(),
                    err.to_string()
                );
            }
            _ => (),
        }
    }

    fn skip_whitespace(&mut self) {
        self.tokens.peek_until(|t| !t.kind().is_whitespace());
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

    pub fn parse_script(&mut self, name: String, source: String) -> AstBody {
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
        let ast = ast_generator.begin_parse(TokenStream::new(tokens)); // parse the tokens.

        return ast;
    }

    pub(crate) fn do_options(&self, tokens: &Vec<Token>) {
        if self.options.semantic_checks == true {
            // do semantic checks
            analyze(tokens.clone());
        }
    }
}
