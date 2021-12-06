use crate::{CompilerOptions, ast::{AstBody, Class, ClassProperty, Statement, Visibility}, lexer::{keyword::KeyWord, token::TokenType}, util::{StreamBuffer, TokenStream}};

use self::context::{Context, ContextStore, SourceOrigin};

pub mod context;

pub struct AstGenerator {
    pub(crate) body: AstBody,
    pub(crate) tokens: TokenStream,
    pub(crate) context: Context
}

impl AstGenerator {
    pub fn new(source: SourceOrigin, id: u64) -> Self {
        AstGenerator {
            body: AstBody::new(),
            tokens: TokenStream::new(Vec::new()),
            context: Context::new(source, id)
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
    pub fn parse_tokens(&mut self, tokens: TokenStream) -> Result<AstBody, String> {
        self.tokens = tokens;

        while !self.tokens.is_eof() {
            self.parse();
        }

        return Ok(self.body);
    }

    fn parse(&mut self) {
        // attempt to parse a statement
        if let Some(stmt) = self.parse_statement() {
            self.body.push_statement(stmt);
            return;
        }
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
                    let declaration_keyword = KeyWord::from_string(&self.tokens.first().unwrap().value().unwrap()).unwrap();

                    if !declaration_keyword.is_declarative() {
                        self.error("Expected a declaration keyword after visibility keyword.".to_string());
                    }

                    return self.parse_declaration(Visibility::from_keyword(keyword), declaration_keyword);
                }
                None
            },
            _ => {
                None
            }
        }
    }

    fn parse_declaration(&mut self, visibility: Visibility, keyword: KeyWord) -> Option<Statement> {
        // checks the type of declaration it is, and parses it accordingly

        match keyword {
            KeyWord::Class => {
                let mut class = Class::new(visibility);
                // we're parsing a class here!
                // we expect a class name next, so we can parse it
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
                None
            },
            _ => None
        };
        None
    }

    fn parse_class_body(&mut self, class: &mut Class) -> Option<Statement> {
        // we're expecting the next token to be a brace.
        // if it's not, we can't parse this class as it's not a body.
        let next = self.tokens.peek().unwrap();

        if !next.kind().is_left_brace() {
            self.error("Expected a brace after class name, this opens a class body.".to_string());
        }

        let mut statements: Vec<Statement> = Vec::new();

        // now we can parse any statement until we reach the end of the class body
        while !self.tokens.first().unwrap().kind().is_right_brace() || !self.tokens.is_eof() {
            let statement = self.parse_statement().unwrap();

            // invalid statements, we need to skip them
            if statement.is_class() {
                self.error("Classes cannot be nested.".to_string());
            }

            if statement.is_type() {
                self.error("Types cannot be declared inside a class.".to_string());
            }

            if statement.is_block() {
                self.error("Blocks cannot be declared inside a class.".to_string());
            }

            // the statement is a immutable class property
            if statement.is_immutable() {
                // get the declaration
                ClassProperty {
                    
                }
            }
        }

        if self.tokens.is_eof() {
            self.error("This class is never closed.".to_string());
        }

        // this should never happen, but in the event it does,
        if !self.tokens.first().unwrap().kind().is_right_brace() {
            self.error("Expected a brace to close the class body.".to_string());
        }

        self.tokens.peek();
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
            astparser: AstGenerator::new(),
        }
    }

    pub fn parse_file(&mut self, )
}