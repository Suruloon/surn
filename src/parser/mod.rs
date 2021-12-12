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

pub mod context;

pub struct AstGenerator {
    pub(crate) body: AstBody,
    pub(crate) tokens: TokenStream,
    pub(crate) context: Context,
}

impl AstGenerator {
    pub fn new(source: SourceOrigin, id: u64) -> Self {
        AstGenerator {
            body: AstBody::new(),
            tokens: TokenStream::new(Vec::new()),
            context: Context::new(source, id),
        }
    }
}

// pub struct ParsingContext {
//     pub context_store: ContextStore,
// }

/// Parses the given token stream into an AST.
/// Returns a Result containing the AST.
/// AST is **not** optimized during this stage, however it is validated.
impl AstGenerator {
    pub fn begin_parse(&mut self, tokens: TokenStream) -> AstBody {
        self.tokens = tokens;

        while !self.tokens.is_eof() {
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

        // we don't know what this is!
        dbg!(self.body.clone());
        create_report!(
            self.context,
            self.tokens.peek().unwrap().range(),
            "Unknown statement".to_string()
        );
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        let first_token = self.tokens.first().unwrap();

        return match first_token.kind() {
            TokenType::KeyWord => {
                let keyword = KeyWord::from_string(&first_token.value().unwrap()).unwrap();

                // check if the keyword is a variable declaration
                if keyword.is_declarative() {
                    self.tokens.peek();
                    return self.parse_declaration(Visibility::Private, keyword);
                }

                if keyword.is_visibility() {
                    // we're going to expect a declaration next, so we can parse it
                    // however, lets consume the visibility keyword token from the stream.
                    self.tokens.peek();

                    // check the next token to see if it's a declaration keyword, if not,
                    // we can't parse this statement
                    // this is an error.
                    if !self.tokens.first().unwrap().kind().is_keyword() {
                        create_report!(
                            self.context,
                            first_token.1,
                            format!("Expected one of \"const\", \"var\", \"class\", \"enum\" or \"interface\" but found: {}", self.tokens.first().unwrap().value().unwrap())
                        );
                    }

                    // we can proceed to check the keyword
                    let declaration_keyword =
                        KeyWord::from_string(&self.tokens.first().unwrap().value().unwrap())
                            .unwrap();

                    if !declaration_keyword.is_declarative() {
                        create_report!(
                            self.context,
                            first_token.1,
                            "One of \"const\", \"var\", \"class\", \"enum\" or \"interface\" must follow a visibilty keyword.".to_string(),
                            "Unexpected visibilty".to_string()
                        );
                    }

                    return self
                        .parse_declaration(Visibility::from_keyword(keyword), declaration_keyword);
                }
                None
            }
            _ => None,
        };
    }

    // fn parse_function(&mut self, visibility: Visibility) -> Option<Statement> {
    // }

    /// This function assumes that the first token is a left parenthesis.
    /// This function will consume the function arguments, and type and return it, leaving the
    /// token stream with a left bracket.
    fn parse_function_inputs(&mut self) -> (Vec<FunctionInput>, Vec<TypeRef>) {
        let mut inputs = Vec::<FunctionInput>::new();
        let mut outputs = Vec::<TypeRef>::new();
        if !self.tokens.first().unwrap().kind().is_left_parenthesis() {
            create_report!(
                self.context,
                self.tokens.first().unwrap().1,
                format!(
                    "Expected a left parenthesis but found: {}",
                    self.tokens.first().unwrap().value().unwrap()
                )
            );
        }
        self.tokens.peek();

        while !self.tokens.first().unwrap().kind().is_right_parenthesis() && !self.tokens.is_eof() {
            let name = self.tokens.peek().unwrap();

            if !name.kind().is_identifier() {
                create_report!(
                    self.context,
                    name.range(),
                    format!("Only function parameters can follow a function name."),
                    format!(
                        "Unexpected {} \"{}\"",
                        name.kind().to_string(),
                        name.value().unwrap()
                    )
                );
            }

            if !self.tokens.first().unwrap().kind().is_colon() {
                create_report!(
                    self.context,
                    name.range(),
                    format!("Expected a colon after a function parameter name."),
                    format!(
                        "Unexpected {} \"{}\"",
                        self.tokens.first().unwrap().kind().to_string(),
                        self.tokens.first().unwrap().value().unwrap()
                    )
                );
            }

            self.tokens.peek();

            let types = self.parse_type_list();

            if self.tokens.first().unwrap().kind().is_comma() {
                self.tokens.peek();
            }

            inputs.push(FunctionInput {
                name: name.value().unwrap().to_string(),
                types: types,
            });
        }

        if !self.tokens.first().unwrap().kind().is_right_parenthesis() {
            create_report!(
                self.context,
                self.tokens.first().unwrap().1,
                format!(
                    "Unexpected token: \"{}\"",
                    self.tokens.first().unwrap().value().unwrap()
                )
            );
        } else {
            self.tokens.peek();
            // we have a return type that is NOT void.
            if self.tokens.first().unwrap().kind().is_colon() {
                self.tokens.peek();
                outputs = self.parse_type_list();
            }
        }

        return (inputs, outputs);
    }

    /// Parses a **Global** declaration.
    /// This is a declaration that is not attached to a class.
    /// However a class body will utilize this function to parse its own `constant` declarations.
    fn parse_declaration(&mut self, visibility: Visibility, keyword: KeyWord) -> Option<Statement> {
        // checks the type of declaration it is, and parses it accordingly
        match keyword {
            KeyWord::Class => {
                let mut class = Class::new(visibility);
                // we're parsing a class here!
                // we expect a class name next, so we can parse it
                // however before that, let's set the contexts flags to reflect this
                self.context.flags = ContextFlag::InClass;
                let name = self.tokens.peek().unwrap();

                if !name.kind().is_identifier() {
                    create_report!(
                        self.context,
                        name.range(),
                        format!("A class name must follow a class declaration."),
                        format!("Unexpected token \"{}\"", name.kind().to_string())
                    );
                }

                // we know the class name now
                class.name = name.value().unwrap().to_string();
                // we need to assign this class a new id!
                class.node_id = self.context.get_next_local_id();

                // we're going to parse the body of the class now.
                let body = self.parse_class_body(&mut class);

                class.body = body;

                // we're done parsing the class body, so we can safely return the class.
                return Some(Statement::Class(class));
            }

            KeyWord::Const => {
                // we're going to parse a constant declaration
                // we expect a variable name next, so we can parse it
                let name = self.tokens.peek().unwrap();

                if !name.kind().is_identifier() {
                    create_report!(
                        self.context,
                        name.range(),
                        format!("A constant name must follow a constant declaration."),
                        format!("Unexpected token \"{}\"", name.kind().to_string())
                    );
                }

                // we know the variable name now
                let variable_name = name.value().unwrap().to_string();
                // we need to assign this variable a new id!
                let variable_id = self.context.get_next_local_id();

                // attempt to parse a type
                // let type_ref: TypeRef = self.parse_type();

                // we need to parse an expression for the variable
                let next = self.tokens.first().unwrap();

                // we're going to parse the expression for the variable now.
                // let expression = self.parse_expression();

                // we're done parsing the expression, so we can safely return the variable.
                return Some(Statement::Immutable(Variable {
                    name: variable_name,
                    node_id: variable_id,
                    visibility: visibility,
                    // this is temprorary
                    assignment: None,
                    // this is also temporary
                    type_ref: TypeRef { context: 0, id: 0 },
                }));
            }
            _ => (),
        };
        return None;
    }

    fn parse_class_property(&mut self, visibility: Visibility) -> ClassProperty {
        // we're going to parse a class property
        // we expect a variable name next, so we can parse it
        let name = self.tokens.peek().unwrap();

        if !name.kind().is_identifier() {
            create_report!(
                self.context,
                name.range(),
                format!("A class property name must follow a class property declaration."),
                format!("Unexpected token \"{}\"", name.kind().to_string())
            );
        }

        // we know the variable name now
        let variable_name = name.value().unwrap().to_string();

        let next = self.tokens.first().unwrap();

        // we need to check if the next token is a type or not
        // if it's the beginning of an assignment, we need to parse the expression.
        if next.kind().is_colon() {
            self.tokens.peek();
            // the next few tokens are a type expression.
            let type_data = self.parse_type_list();
            // we're going to parse the expression for the variable now.
            let expression: Option<Expression> = {
                // get the next token
                let next = self.tokens.peek().unwrap();
                if next.kind().is_assignment() {
                    create_report!(
                        self.context,
                        next.range(),
                        format!("Expected an assignment or statement end after a class property declaration."),
                        format!("Unexpected token \"{}\"", next.kind().to_string())
                    );
                    // self.parse_expression()
                } else if next.kind().is_statement_end() {
                    None
                } else {
                    create_report!(
                        self.context,
                        next.range(),
                        format!("Expected an assignment or statement end after a class property declaration."),
                        format!("Unexpected token \"{}\"", next.kind().to_string())
                    );
                }
            };

            // we're done parsing the expression, so we can safely return the variable.
            return ClassProperty {
                name: variable_name,
                visibility: visibility,
                // this is temprorary
                value: expression,
                // this is also temporary
                type_ref: TypeRef { context: 0, id: 0 },
            };
        }
        // we're going to parse the expression for the variable now.
        // let expression = self.parse_expression();

        // we're done parsing the expression, so we can safely return the variable.
        return ClassProperty {
            name: variable_name,
            visibility: visibility,
            // this is also temporary
            type_ref: TypeRef { context: 0, id: 0 },
            value: None,
        };
    }

    fn parse_class_method(&mut self, visibility: Visibility) -> Statement {
        // function statement lulz
        let name = self.tokens.peek().unwrap();

        if !name.kind().is_identifier() {
            create_report!(
                self.context,
                name.range(),
                format!("A class method name must follow a class method declaration."),
                format!("Unexpected token \"{}\"", name.kind().to_string())
            );
        }

        // we know the method name now
        let method_name = name.value().unwrap().to_string();
        // we need to assign this method a new id!
        let method_id = self.context.get_next_local_id();

        let args = self.parse_function_inputs();

        dbg!(&args);

        // get the scope for the method
        let scope = self.parse_scope();

        Statement::Function(Function {
            name: method_name,
            node_id: method_id,
            body: scope,
            visibility: visibility,
            inputs: args.0,
            outputs: args.1,
        })
    }

    fn parse_class_body(&mut self, class: &mut Class) -> Vec<Statement> {
        // we're expecting the next token to be a brace.
        // if it's not, we can't parse this class as it's not a body.
        let next = self.tokens.peek().unwrap();

        if !next.kind().is_left_brace() {
            create_report!(
                self.context,
                next.range(),
                format!("A class body must follow a class declaration. You should open a body with a brace here."),
                format!("Unexpected token \"{}\"", next.kind().to_string())
            );
        }

        let mut statements: Vec<Statement> = Vec::new();

        // now we can parse any statement until we reach the end of the class body
        // this is the only special case where we need to specially parse a body or block of code
        // because of how php works :rage:
        while !self.tokens.first().unwrap().kind().is_right_brace() && !self.tokens.is_eof() {
            let next_token = self.tokens.first().unwrap();
            let statement: Statement = match next_token.kind() {
                TokenType::KeyWord => {
                    let keyword = KeyWord::from_string(&next_token.value().unwrap()).unwrap();

                    if keyword.is_declarative() {
                        self.tokens.peek();
                        // check what kind of declaration it is
                        // if its immutable, we can parse it, if it's not, we can't 😔
                        if keyword == KeyWord::Const {
                            self.parse_declaration(Visibility::Private, keyword)
                                .unwrap()
                        } else {
                            create_report!(
                                self.context,
                                next_token.range(),
                                format!("This keyword is useless in class context."),
                                format!(
                                    "Useless Keyword \"{}\"",
                                    next_token.value().unwrap().to_string()
                                )
                            );
                        }
                    } else if keyword.is_visibility() {
                        // we're going to expect a identifier OR a declaration next, so we can parse it
                        // however, lets consume the visibility keyword token from the stream.
                        self.tokens.peek();

                        let next_token = self.tokens.first().unwrap();

                        // check if the next token is a function keyword or not
                        if next_token.kind().is_keyword() {
                            if KeyWord::Function
                                == KeyWord::from_string(&next_token.value().unwrap()).unwrap()
                            {
                                // this a class method!
                                self.tokens.peek();
                                self.parse_class_method(Visibility::from_keyword(keyword))
                            } else if keyword.is_declarative() {
                                dbg!(&keyword);
                                self.tokens.peek();
                                // check what kind of declaration it is
                                // if its immutable, we can parse it, if it's not, we can't 😔
                                if keyword == KeyWord::Const {
                                    self.parse_declaration(Visibility::Private, keyword)
                                        .unwrap()
                                } else {
                                    create_report!(
                                        self.context,
                                        next_token.range(),
                                        format!("This keyword is useless in class context."),
                                        format!(
                                            "Useless Keyword \"{}\"",
                                            next_token.value().unwrap().to_string()
                                        )
                                    );
                                }
                            } else {
                                create_report!(
                                    self.context,
                                    next_token.range(),
                                    format!("One of \"const\", \"var\", \"class\", \"enum\" or \"interface\" must follow a visibilty keyword."),
                                    format!("Unexpected {}: \"{}\"", next_token.kind().to_string(), next_token.value().unwrap().to_string())
                                );
                            }
                        } else {
                            let property =
                                self.parse_class_property(Visibility::from_keyword(keyword));

                            class.properties.push(property);
                            continue;
                        }
                    } else {
                        create_report!(
                            self.context,
                            next_token.range(),
                            format!("One of \"const\", \"var\", \"class\", \"enum\" or \"interface\" must follow a visibilty keyword."),
                            format!("Unexpected {}: \"{}\"", next_token.kind().to_string(), next_token.value().unwrap().to_string())
                        );
                    }
                }
                _ => {
                    create_report!(
                        self.context,
                        next_token.range(),
                        format!(
                            "Illegal token in class body \"{}\"",
                            next_token.kind().to_string()
                        )
                    );
                }
            };

            // classes **can** have functions and properties, so we can parse them
            statements.push(statement);
        }

        if self.tokens.is_eof() {
            create_report!(
                self.context,
                self.tokens.prev().unwrap().range(),
                format!("This class is never closed."),
                format!("Unexpected end of file")
            );
        }

        // this should never happen, but in the event it does,
        if !self.tokens.first().unwrap().kind().is_right_brace() {
            create_report!(
                self.context,
                self.tokens.prev().unwrap().range(),
                format!("This class is never closed."),
                format!("Unexpected end of file")
            );
        }

        dbg!(self.tokens.peek().unwrap());
        return statements;
    }

    fn parse_type_list(&mut self) -> Vec<TypeRef> {
        let mut types: Vec<Type> = Vec::new();

        while !self.tokens.is_eof() {
            // check the next token
            let tk = self.tokens.first().unwrap();
            match tk.kind() {
                TokenType::Identifier => {
                    types.push(Type {
                        name: tk.value().unwrap(),
                        kind: TypeKind::Ref(TypeRef {
                            context: self.context.origin,
                            id: 0,
                        }),
                        id: 0,
                    });
                    self.tokens.peek();
                }
                TokenType::Operator => {
                    if tk.value().unwrap() == "|".to_string() {
                        self.tokens.peek();
                        continue;
                    } else {
                        create_report!(
                            self.context,
                            tk.range(),
                            format!(
                                "Unexpected operator \"{}\" in type statement",
                                tk.kind().to_string()
                            )
                        );
                    }
                }
                _ => break,
            }
        }

        types
            .into_iter()
            .map(|t| self.context.types.make_type(t))
            .collect()
    }

    fn parse_scope(&mut self) -> Vec<Statement> {
        // we're expecting the next token to be a brace.
        let tk = self.tokens.peek().unwrap();
        let mut statements: Vec<Statement> = Vec::new();

        if tk.kind().is_left_brace() {
            // this is a bit hacky, but bare with me.
            // we are going to parse a statement until we find a right brace.
            while !self.tokens.first().unwrap().kind().is_right_brace() && !self.tokens.is_eof() {
                println!("Inside scope parsing loop");
                if let Some(stmt) = self.parse_statement() {
                    statements.push(stmt);
                }
                self.tokens.peek().unwrap();
            }

            if self.tokens.is_eof() || !self.tokens.first().unwrap().kind().is_right_brace() {
                create_report!(
                    self.context,
                    self.tokens.prev().unwrap().range(),
                    format!("This scope is never closed."),
                    format!("Unexpected end of file")
                );
            }

            self.tokens.peek().unwrap();

            return statements;
        } else {
            create_report!(
                self.context,
                self.tokens.prev().unwrap().range(),
                format!("Expected a left brace to open this scope body."),
                format!("Unexpected token \"{}\"", tk.kind().to_string())
            );
        }
    }

    fn parse_keyword() -> Option<Statement> {
        None
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
