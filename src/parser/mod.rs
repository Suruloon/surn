use std::process;

use crate::{
    ast::{
        Array, AstBody, Call, Class, ClassProperty, Expression, Function, FunctionInput, Literal,
        MemberListNode, NewCall, Object, ObjectProperty, Operation, Statement, Variable,
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
        dbg!("At line.");
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
        dbg!("At line.");
        process::exit(1);
    };
}

pub struct AstGenerator {
    pub(crate) body: AstBody,
    pub(crate) tokens: TokenStream,
    pub(crate) context: Context,
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

        if self
            .tokens
            .first()
            .unwrap_or(Token(TokenType::Whitespace, 0..1, None))
            .kind()
            .is_whitespace()
        {
            self.tokens.peek();
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
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
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

    /// Parses an expression.
    /// For example:
    /// - `5`
    /// - `x`
    /// - `x + 5`
    /// - `x + 5 * y`
    fn parse_expression(&mut self) -> Option<Expression> {
        // parse a call expression
        if let Some(call_expr) = self.parse_call_expression() {
            return Some(Expression::Call(call_expr));
        }

        // parse a member expression
        if let Some(member_expr) = self.parse_member_expression() {
            return Some(Expression::Member(member_expr));
        }

        // parse a new expression
        if let Some(new_expr) = self.parse_new_expression() {
            return Some(Expression::New(new_expr));
        }

        // parse an array
        if let Some(array_expr) = self.parse_array_expression() {
            return Some(Expression::Array(array_expr));
        }

        // parse a statement expression
        // this needs to be before object parsing because
        // object expressions will assume a block check has already taken place.
        if let Some(statement_expr) = self.parse_statement() {
            return Some(Expression::Statement(Box::new(statement_expr)));
        }

        // parse an object
        if let Some(object_expr) = self.parse_object_expression() {
            return Some(Expression::Object(object_expr));
        }

        // parse an operator expression
        if let Some(operator_expr) = self.parse_operator_expression() {
            return Some(Expression::Operation(operator_expr));
        }

        // parse a literal expression
        if let Some(literal_expr) = self.parse_literal_expression() {
            return Some(Expression::Literal(literal_expr));
        }

        return None;
    }

    fn parse_call_expression(&mut self) -> Option<Call> {
        // parse a call expression
        if let Some(identifier) = self.tokens.first_if(|t| t.kind().is_identifier()) {
            // we have an identifier, we need to try to parse function arguments now.
            if let Some(args) = self.parse_function_call_inputs() {
                // This is definitely a function call.
                return Some(Call::new(identifier.value().unwrap(), args));
            } else {
                // This probably isn't a function call.
                return None;
            }
        }

        return None;
    }

    fn parse_member_expression(&mut self) -> Option<MemberListNode> {
        // parse a member expression
        if let Some(identifier) = self.tokens.first_if(|t| t.kind().is_identifier()) {
            // we have an identifier, we need to try to parse member expressions now.
            // we need to verify that this is a member expression
            // we need to check if the next token is a period
            if let Some(_) = self
                .tokens
                .second_if(|t| t.kind().is_operator() && (t.value().unwrap() == ".".to_string()))
            {
                self.tokens.peek_inc(2);
                // we have a period, we need to parse a member expression
                // we need to parse a member expression
                if let Some(member_expr) = self.parse_expression() {
                    // we have a member expression, we need to create a member list node
                    return Some(MemberListNode::new(member_expr, identifier.clone()));
                } else {
                    // we don't have a member expression, we need to report an error
                    create_report!(
                        self.context,
                        self.tokens.first().unwrap().range(),
                        "Expected an expression to follow a property member.".to_string(),
                        "An expression was expected here.".to_string()
                    );
                }
            } else {
                // we don't have a period, this is probably not a member expression
                return None;
            }
        }

        return None;
    }

    fn parse_new_expression(&mut self) -> Option<NewCall> {
        if let Some(_) = self
            .tokens
            .first_if(|t| t.kind().is_keyword() && (t.value().unwrap() == "new".to_string()))
        {
            // we have a new keyword, we need to parse a name.
            if let Some(name) = self.tokens.second_if(|t| t.kind().is_identifier()) {
                // we have a name, we need to parse a function call inputs.
                if let Some(args) = self.parse_function_call_inputs() {
                    // we have a function call inputs, we need to create a new call.
                    return Some(NewCall::new(name.value().unwrap(), args));
                } else {
                    // we don't have a function call inputs, we need to report an error.
                    create_report!(
                        self.context,
                        self.tokens.first().unwrap().range(),
                        "Expected a function call inputs to follow a new expression.".to_string(),
                        "Function inputs expected here.".to_string()
                    );
                }
            } else {
                // we don't have a name, we need to report an error.
                create_report!(
                    self.context,
                    self.tokens.first().unwrap().range(),
                    "Expected a name to follow a new expression.".to_string(),
                    "A name was expected here.".to_string()
                );
            }
        }
        return None;
    }

    fn parse_array_expression(&mut self) -> Option<Array> {
        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_left_bracket()) {
            // inside array
            let mut elements: Vec<Expression> = Vec::new();
            while !self.tokens.is_eof() {
                self.skip_whitespace_err("Array's must be closed.");
                if let Some(element) = self.parse_expression() {
                    // we have an expression, we need to parse a comma
                    self.skip_whitespace_err("Array's must be closed.");
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                        elements.push(element);
                    } else {
                        // ok, check if the next token is a right bracket, if so, we're done.
                        // otherwise error
                        self.skip_whitespace_err("Array's must be closed.");
                        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_bracket()) {
                            // we have a right bracket, we can return the inputs
                            elements.push(element);
                            return Some(Array::new(elements, None));
                        } else {
                            create_report!(
                                self.context,
                                self.tokens.first().unwrap().range(),
                                "A comma is required to seperate array elements.".to_string(),
                                "A comma is expected here.".to_string()
                            );
                        }
                    }
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_bracket()) {
                    // end of array
                    return Some(Array::new(elements, None));
                } else {
                    // we don't have an expression, we need to report an error.
                    create_report!(
                        self.context,
                        self.tokens.first().unwrap().range(),
                        "Expected an expression to follow an array element.".to_string(),
                        format!(
                            "Unexpected Token: {}",
                            self.tokens.first().unwrap().kind().to_string()
                        )
                    );
                }
            }
        }
        return None;
    }

    fn parse_object_expression(&mut self) -> Option<Object> {
        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_left_brace()) {
            // this is definitely an object body.
            let mut object: Object = Object::empty();

            while !self.tokens.is_eof() {
                // purge whitespace.
                self.skip_whitespace_err("Object body must be closed.");
                if let Some(property) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                    // the property name was found, now we need to parse a colon.
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_colon()) {
                        // we have a colon, we need to parse an expression.
                        self.skip_whitespace_err("Object body must be closed.");
                        if let Some(expression) = self.parse_expression() {
                            // we have an expression, we need to add the property to the object.
                            let prop = ObjectProperty::new(property.value().unwrap(), expression);

                            // check if we have a comma, if so, we need to parse another property.
                            // otherwise we need to check if we have a right brace, if so, we're done.
                            if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                                // we have a comma, we need to parse another property.
                                object.properties.push(prop);
                            } else {
                                // check for a right brace, if so, we're done.
                                self.skip_whitespace_err("Object body must be closed.");
                                if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_brace())
                                {
                                    // we have a right brace, we're done.
                                    object.properties.push(prop);
                                    return Some(object);
                                } else {
                                    // we don't have a right brace, we need to report an error.
                                    create_report!(
                                        self.context,
                                        self.tokens.first().unwrap().range(),
                                        "Expected a right brace to close an object body."
                                            .to_string(),
                                        "A right brace was expected here.".to_string()
                                    );
                                }
                            }
                        } else {
                            // we don't have an expression, we need to report an error.
                            create_report!(
                                self.context,
                                self.tokens.first().unwrap().range(),
                                "Expected an expression to follow a property.".to_string(),
                                "An expression was expected here.".to_string()
                            );
                        }
                    } else {
                        // we don't have a colon, we need to report an error.
                        create_report!(
                            self.context,
                            self.tokens.first().unwrap().range(),
                            "Expected a colon to follow a property name.".to_string(),
                            "A colon was expected here.".to_string()
                        );
                    }
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_brace()) {
                    // end of object
                    return Some(object);
                } else {
                    // we don't have an object property, we need to report an error.
                    create_report!(
                        self.context,
                        self.tokens.first().unwrap().range(),
                        "Expected an object property to follow an object element.".to_string(),
                        "An object property was expected here.".to_string()
                    );
                }
            }
        }
        return None;
    }

    fn parse_operator_expression(&mut self) -> Option<Operation> {
        None
    }

    fn parse_literal_expression(&mut self) -> Option<Literal> {
        // we have a literal, we need to parse a value.
        // a literal is either a string, number, boolean or null
        // either way we need to check if the next token is a identifier.
        if let Some(v) = self
            .tokens
            .peek_if(|t| t.kind().is_identifier() || t.kind().is_number() || t.kind().is_string())
        {
            return Some(Literal::new(v.value().unwrap(), None));
        } else {
            return None;
        }
    }

    /// parses function inputs (aka arguments)
    fn parse_function_call_inputs(&mut self) -> Option<Vec<Expression>> {
        // parse a function input
        // we need to check for a parenthesis
        if let Some(_) = self.tokens.second_if(|t| t.kind().is_left_parenthesis()) {
            // ok we have a parenthesis!
            // lets peek to the next token now.
            self.tokens.peek_inc(2);
            // we're inside a parenthesis, we need to parse an expression now.
            let mut inputs: Vec<Expression> = Vec::new();
            while !self.tokens.is_eof() {
                // we need to parse an expression
                self.skip_whitespace_err("Function arguments must be closed.");

                if let Some(expr) = self.parse_expression() {
                    // we have an expression, we need to parse a comma
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                        inputs.push(expr);
                    } else {
                        // ok, check if the next token is a parenthises, if so, we're done.
                        // otherwise error
                        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_parenthesis()) {
                            // we have a right parenthesis, we can return the inputs
                            return Some(inputs);
                        } else {
                            create_report!(
                                self.context,
                                self.tokens.first().unwrap().range(),
                                "Expected a comma to follow a function input.".to_string(),
                                "A comma is expected here.".to_string()
                            );
                        }
                    }
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_parenthesis()) {
                    // we have a right parenthesis, we can return the inputs
                    return Some(inputs);
                } else {
                    // we don't have an expression, we need to report an error
                    create_report!(
                        self.context,
                        self.tokens.first().unwrap().range(),
                        "Expected an expression to follow a function input.".to_string(),
                        "An expression is expected here.".to_string()
                    );
                }
            }

            create_report!(
                self.context,
                self.tokens.first().unwrap().range(),
                "Expected an expression to follow a function input.".to_string(),
                "An expression is expected here.".to_string()
            );
        }

        return None;
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
        self.tokens.peek_until(|t| {
            if t.kind().is_whitespace() {
                return false;
            } else {
                return true;
            }
        });
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
