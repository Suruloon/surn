use crate::{
    ast::{AstBody, Class, ClassProperty, Statement, Variable, Visibility},
    lexer::{
        analysis::analyze,
        keyword::KeyWord,
        token::{Token, TokenType},
        tokenizer::tokenize,
    },
    types::TypeRef,
    util::{StreamBuffer, TokenStream},
    CompilerOptions,
};

use self::context::{Context, ContextFlag, ContextStore, SourceOrigin};

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
        self.error(format!(
            "Unexpected token: {}",
            self.tokens.first().unwrap().value().unwrap()
        ));
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
                        self.error("Expected a declaration keyword after visibility keyword, but found something different.".to_string());
                    }

                    // we can proceed to check the keyword
                    let declaration_keyword =
                        KeyWord::from_string(&self.tokens.first().unwrap().value().unwrap())
                            .unwrap();

                    if !declaration_keyword.is_declarative() {
                        self.error(
                            "Expected a declaration keyword after visibility keyword.".to_string(),
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
                    self.error("Expected a class name after class keyword.".to_string());
                }

                // we know the class name now
                class.name = name.value().unwrap().to_string();
                // we need to assign this class a new id!
                class.node_id = self.context.get_next_local_id();

                // we need to parse an entire body of code for the class
                let next = self.tokens.first().unwrap();

                // we're going to parse the body of the class now.
                let body = self.parse_class_body(&mut class);

                // we're done parsing the class body, so we can safely return the class.
                return Some(Statement::Class(class));
            }

            KeyWord::Const => {
                // we're going to parse a constant declaration
                // we expect a variable name next, so we can parse it
                let name = self.tokens.peek().unwrap();

                if !name.kind().is_identifier() {
                    self.error("Expected a variable name after const keyword.".to_string());
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
                    type_ref: TypeRef {
                        context: 0,
                        node: 0,
                    },
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
            self.error("Expected a variable name.".to_string());
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
        return ClassProperty {
            name: variable_name,
            visibility: visibility,
            // this is also temporary
            type_ref: TypeRef {
                context: 0,
                node: 0,
            },
            value: None,
        };
    }

    fn parse_class_body(&mut self, class: &mut Class) -> Vec<Statement> {
        // we're expecting the next token to be a brace.
        // if it's not, we can't parse this class as it's not a body.
        let next = self.tokens.peek().unwrap();

        if !next.kind().is_left_brace() {
            self.error("Expected a brace after class name, this opens a class body.".to_string());
        }

        let mut statements: Vec<Statement> = Vec::new();

        // now we can parse any statement until we reach the end of the class body
        // this is the only special case where we need to specially parse a body or block of code
        // because of how php works :rage:
        while !self.tokens.first().unwrap().kind().is_right_brace() || !self.tokens.is_eof() {
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
                            self.error("This keyword is useless in this context.".to_string());
                            // this is unreachable, but whatever.
                            continue;
                        }
                    } else if keyword.is_visibility() {
                        // we're going to expect a identifier OR a declaration next, so we can parse it
                        // however, lets consume the visibility keyword token from the stream.
                        self.tokens.peek();

                        let property = self.parse_class_property(Visibility::from_keyword(keyword));

                        class.properties.push(property);
                        continue;
                    } else {
                        self.tokens.peek().unwrap();
                        continue;
                    }
                }
                _ => {
                    self.tokens.peek().unwrap();
                    continue;
                }
            };

            // classes **can** have functions and properties, so we can parse them
            statements.push(statement);
        }

        if self.tokens.is_eof() {
            self.error("This class is never closed.".to_string());
        }

        // this should never happen, but in the event it does,
        if !self.tokens.first().unwrap().kind().is_right_brace() {
            self.error("Expected a brace to close the class body.".to_string());
        }

        self.tokens.peek();
        return statements;
    }

    fn parse_keyword() -> Option<Statement> {
        None
    }

    fn error(&self, reason: String) {
        panic!("{}", reason);
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
