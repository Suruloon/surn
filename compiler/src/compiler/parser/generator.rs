// Home of the Surn Parser.
use std::ops::Range;

use crate::compiler::{
    ast::{
        ops::AnyOperation, Array, AstBody, Call, Class, ClassAllowedStatement, ClassBody,
        ClassProperty, Expression, Function, FunctionInput, Literal, MemberListNode, MemberLookup,
        Namespace, NewCall, Object, ObjectProperty, Operation, Path, Return, Statement, Static,
        Variable, Visibility,
    },
    ast::{
        types::{BuiltInType, TypeDefinition, TypeKind, TypeParam, TypeReference, TypeUnion},
        Node,
    },
    lexer::{
        keyword::KeyWord,
        token::{Token, TokenType},
    },
};

use super::{
    context::{Context, SourceOrigin},
    ParserError,
};
use crate::util::{StreamBuffer, TokenStream};

pub fn combine_ranges(ranges: Vec<Range<usize>>) -> Range<usize> {
    let mut start = 0;
    let mut end = 0;
    for range in ranges.iter() {
        if range.start < start {
            start = range.start;
        }
        if range.end > end {
            end = range.end;
        }
    }
    start..end
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

    pub fn begin_parse(&mut self, tokens: TokenStream) -> Result<AstBody, ParserError> {
        self.tokens = tokens;

        while !self.tokens.is_eof() {
            self.skip_whitespace();
            self.parse()?;
        }

        return Ok(self.body.clone());
    }

    fn parse(&mut self) -> Result<(), ParserError> {
        // attempt to parse a statement
        let start = {
            if let Some(token) = self.tokens.first() {
                token.range()
            } else {
                Range { start: 0, end: 0 }
            }
        };

        if let Some(stmt) = self.parse_statement()? {
            self.body.push_node(Node::new(
                stmt.into(),
                start,
                self.tokens.prev().unwrap().range(),
            ));
            return Ok(());
        }

        if let Some(left) = self.parse_expression()? {
            self.body.push_node(Node::new(
                left.into(),
                start,
                self.tokens.prev().unwrap().range(),
            ));
            return Ok(());
        }

        if self
            .tokens
            .first()
            .unwrap_or(Token(TokenType::Whitespace, 0..1, None))
            .kind()
            .is_whitespace()
        {
            self.tokens.peek();
            return Ok(());
        }

        // we don't know what this is!
        // the only body we can have is a statement or an expression
        return Err(ParserError::new(
            format!("Missing a valid statement or expression in global scope."),
            "Unable to proceed parsing. A known token was unexpected at this time.".to_string(),
            combine_ranges(vec![start, self.tokens.first().unwrap().range()]),
            self.body.clone(),
            None,
        ));
    }

    /// A statement can be a variable declaration, function declaration, class declaration, etc.
    fn parse_statement(&mut self) -> Result<Option<Statement>, ParserError> {
        if let Some(namespace) = self.parse_namespace()? {
            return Ok(Some(Statement::Namespace(namespace)));
        }

        // Try to parse a static statement (this is obsolete in global context, but can exist)
        // this is transpiled to a GLOBALS class.
        if let Some(stmt) = self.parse_static()? {
            return Ok(Some(stmt));
        }

        // try to parse a mutable or constant variable.
        if let Some((var, constant)) = self.parse_variable()? {
            if constant {
                return Ok(Some(Statement::Const(var)));
            } else {
                return Ok(Some(Statement::Var(var)));
            }
        }

        // try to parse a function declaration
        if let Some(func) = self.parse_function()? {
            return Ok(Some(Statement::Function(func)));
        }

        if let Some(class) = self.parse_class()? {
            return Ok(Some(Statement::Class(class)));
        }

        return Ok(None);
    }

    fn parse_namespace(&mut self) -> Result<Option<Namespace>, ParserError> {
        if let Some(_) = self
            .tokens
            .peek_if(|t| t.kind().is_keyword() && (t.kind().as_keyword() == KeyWord::Namespace))
        {
            let start = self.tokens.first().unwrap().range();
            let mut path: Vec<String> = Vec::new();
            self.skip_whitespace();
            if let Some(name) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                // we need to parse a path now.
                loop {
                    self.skip_whitespace();
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_backslash()) {
                        if let Some(ident) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                            path.push(ident.value().unwrap());
                        } else {
                            return Err(ParserError::new(
                                format!("Expected identifier after backslash."),
                                format!("Namespace sub-directories must always contain a valid identifier. A valid identifier is an expression that contains a letter, number preceeding any letter, or an underscore."),
                                combine_ranges(vec![start, self.tokens.first().unwrap().range()]),
                                self.body.clone(),
                                None
                            ));
                        }
                    } else if let Some((amt, _)) = self
                        .tokens
                        .find_after(|t| t.kind().is_left_brace(), |t| t.kind().is_whitespace())
                    {
                        self.tokens.peek_inc(amt);
                        if let Some(block) = self.parse_block()? {
                            if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                                return Ok(Some(Namespace {
                                    path: Path::from(name.value().unwrap(), path),
                                    body: Some(Box::new(Statement::Block(block))),
                                }));
                            } else {
                                return Err(ParserError::new(
                                    "A semi-colon was expected.".to_string(),
                                    format!("If a namespace is opening a block, it must always end with a semicolon to identify the end of the namespace."),
                                    self.tokens.first().unwrap().range(),
                                    self.body.clone(),
                                    None
                                ));
                            }
                        } else {
                            return Err(ParserError::new(
                                "Expected block after namespace with opening brace.".to_string(),
                                format!("A statement block must always have an end. Identify the end with a curly brace }}."),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None
                            ));
                        }
                    } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                        return Ok(Some(Namespace {
                            path: Path::from(name.value().unwrap(), path),
                            body: None,
                        }));
                    } else {
                        return Err(ParserError::new(
                            "Unable to parse namespace path.".to_string(),
                            format!("This statement is incomplete. A valid namespace statement contains the namespace keyword followed by a path preceeded with a backslash. A path is a series of identifiers seperated by backslashes."),
                            combine_ranges(vec![start, self.tokens.first().unwrap().range()]),
                            self.body.clone(),
                            Some(format!(
                                "Unexpected token: {}",
                                self.tokens.peek().unwrap().kind().to_string()
                            ))
                        ));
                    }
                }
            } else {
                return Err(ParserError::new(
                    "Expected a namespace name.".to_string(),
                    format!("A namespace must always contain a path. A path is a series of identifiers seperated by backslashes."),
                    combine_ranges(vec![start, self.tokens.first().unwrap().range()]),
                    self.body.clone(),
                    None
                ));
            }
        }
        return Ok(None);
    }

    /// Parses a static statement (if plausible).
    /// A static statement can only be declared in classes and will be checked after initial parsing.
    fn parse_static(&mut self) -> Result<Option<Statement>, ParserError> {
        let start = self.tokens.first().unwrap().range();
        // We actually can't parse visibility here, because a static statement may not exist, however,
        // we will parse it later, if visibility is present.
        if let Some(_) = self
            .tokens
            .first_if(|t| t.kind().is_keyword() && t.kind().as_keyword().is_visibility())
        {
            // We have a keyword however we need to make sure we have a static keyword next.
            if let Some(_) = self
                .tokens
                .second_if(|t| t.kind().is_keyword() && (t.kind().as_keyword() == KeyWord::Static))
            {
                let visibility = self.parse_visibility()?.unwrap();
                self.tokens.peek();
                self.skip_whitespace();
                // We have a static keyword, so we can parse the rest of the statement.
                if let Some(stmt) = self.parse_statement()? {
                    return Ok(Some(Statement::Static(Static::new(visibility, stmt))));
                } else {
                    return Err(ParserError::new(
                        format!("A statement was expected here."),
                        format!("Expected a statement after a static keyword, but found none."),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            } else {
                return Ok(None);
            }
        }

        // check if we have a static keyword!
        if let Some(_) = self
            .tokens
            .first_if(|t| t.kind().is_keyword() && (t.kind().as_keyword() == KeyWord::Static))
        {
            self.tokens.peek();
            self.skip_whitespace();
            // We have a static keyword, so we can parse the rest of the statement.
            if let Some(stmt) = self.parse_statement()? {
                return Ok(Some(Statement::Static(Static::new(
                    Visibility::Private,
                    stmt,
                ))));
            } else {
                return Err(ParserError::new(
                    format!("A statement was expected here."),
                    format!("Expected a statement after a static keyword, but found none."),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        } else {
            return Ok(None);
        }
    }

    /// Parses a variable declaration (if plausible)
    ///
    /// For example:
    /// - `var x = 5`
    /// - `const x = 5`
    fn parse_variable(&mut self) -> Result<Option<(Variable, bool)>, ParserError> {
        // check for visibility
        let visibility = self.parse_visibility()?.unwrap_or(Visibility::Private);
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
            self.skip_whitespace_err("A variable name was expected but none was found.")?;

            // check if the next token is an indentifier
            if let Some(identifier) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                let mut type_node: Option<TypeKind> = None;
                // (&mut type_node).take();

                // token is an identifier!
                // we need to check if a colon follows, if so, we need to parse a type, otherwise we can skip
                // the type checking and just parse the variable
                if let Some(_) = self.tokens.peek_if(|t| t.kind().is_colon()) {
                    // now parse a type statement.
                    if let Some(type_smt) = self.parse_type_kind()? {
                        type_node = Some(type_smt);
                    } else {
                        return Err(ParserError::new(
                            "A type statement is expected here.".to_string(),
                            "Expected type statement to follow a variable declaration with a colon.".to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None
                        ));
                    }
                } else {
                    type_node = None;
                }

                // we now need an assignment operator
                self.skip_whitespace_err("An operator was expected but none was found.")?;

                // check for an "equals" operator
                if let Some(_) = self
                    .tokens
                    .peek_if(|t| t.kind().is_operator() && (t.value().unwrap() == "=".to_string()))
                {
                    // we have an equals operator!
                    // we need to parse an expression
                    self.skip_whitespace_err("An expression was expected but none was found.")?;
                    if let Some(expr) = self.parse_expression()? {
                        // we have an expression!
                        // we need to parse a semicolon
                        self.skip_whitespace_err("A semicolon was expected but none was found.")?;
                        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                            return Ok(Some((
                                Variable::new(
                                    identifier.value().unwrap(),
                                    type_node,
                                    visibility,
                                    Some(expr),
                                ),
                                is_constant,
                            )));
                        } else {
                            return Err(ParserError::new(
                                "A semicolon is expected here.".to_string(),
                                "Expected a semicolon to follow a variable declaration."
                                    .to_string(),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None,
                            ));
                        }
                    } else {
                        return Err(ParserError::new(
                            "An expression is expected here.".to_string(),
                            "Expected an expression to follow a variable declaration.".to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                } else {
                    // variables **can** be uninitialized
                    // we need to check if the next token is an end of statement
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                        // we have an end of statement!
                        // we can return a variable declaration
                        return Ok(Some((
                            Variable::new(identifier.value().unwrap(), type_node, visibility, None),
                            is_constant,
                        )));
                    } else {
                        // we don't have an end of statement!
                        // we need to report an error
                        return Err(ParserError::new(
                            "A semi-colon is expected here.".to_string(),
                            "Expected an end of statement to follow an uninitialized declaration."
                                .to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                }
            } else {
                return Err(ParserError::new(
                    format!(
                        "Unexpected token: \"{}\"",
                        self.tokens.first().unwrap().kind().to_string()
                    ),
                    "A name must follow a variable declaration".to_string(),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        } else {
            return Ok(None);
        }
    }

    /// Parses a function declaration
    ///
    /// For example:
    /// - `function foo() {}`
    /// - `function foo(x, y) {}`
    /// - `function foo(x, y): int {}`
    fn parse_function(&mut self) -> Result<Option<Function>, ParserError> {
        if let Some(_) = self
            .tokens
            .peek_if(|t| t.kind().is_keyword() && (t.kind().as_keyword() == KeyWord::Function))
        {
            let _ = self.parse_visibility()?.unwrap_or(Visibility::Private);
            let mut name: Option<String> = None;
            self.skip_whitespace_err("A function input list was expected but none was found.")?;
            if let Some(n) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                // we have a function name.
                // we need to parse the input list
                name = n.value();
            }

            // we need to parse the input list
            self.skip_whitespace_err("A function input list was expected but none was found.")?;
            if let Some((inputs, outputs)) = self.parse_function_inputs()? {
                // we need a block now.
                self.skip_whitespace_err("A block was expected but none was found.")?;
                if let Some(block) = self.parse_block()? {
                    return Ok(Some(Function {
                        name,
                        inputs,
                        outputs,
                        body: Box::new(Statement::Block(block)),
                        visibility: Visibility::Public,
                        node_id: 0,
                    }));
                } else {
                    return Err(ParserError::new(
                        "A block is expected here.".to_string(),
                        "Expected a block to follow a function declaration.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            } else {
                return Err(ParserError::new(
                    "A function input list is expected here.".to_string(),
                    "Expected a function input list to follow a function declaration.".to_string(),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        }
        return Ok(None);
    }

    fn parse_function_inputs(
        &mut self,
    ) -> Result<Option<(Vec<FunctionInput>, Option<TypeKind>)>, ParserError> {
        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_left_parenthesis()) {
            let mut inputs: Vec<FunctionInput> = Vec::new();
            while !self.tokens.is_eof() {
                self.skip_whitespace_err("Function declaration arguments must be closed.")?;
                if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_parenthesis()) {
                    // we can't actually return here because we still need to parse the function body
                    // as well as the return type
                    break;
                } else if let Some(param_name) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                    // we have an identifier!
                    // we need to check if a colon follows, if so, we need to parse a type, otherwise we can skip
                    // the type checking and just parse the variable
                    self.skip_whitespace_err(
                        "Expected a type statement after a function argument declaration.",
                    )?;
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_colon()) {
                        // now parse a type statement.
                        self.skip_whitespace();
                        if let Some(type_smt) = self.parse_type_kind()? {
                            // we have a type!
                            // we need to parse a comma
                            self.skip_whitespace_err("A comma was expected but none was found.")?;
                            if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                                // we have a comma!
                                // we need to parse another argument
                                inputs.push(FunctionInput::new(
                                    param_name.value().unwrap_or("".to_string()),
                                    Some(type_smt),
                                ));
                            } else {
                                // we don't have a comma!
                                // we should check if a right parentises follows now
                                if let Some(_) =
                                    self.tokens.peek_if(|t| t.kind().is_right_parenthesis())
                                {
                                    inputs.push(FunctionInput::new(
                                        param_name.value().unwrap(),
                                        Some(type_smt),
                                    ));
                                    break;
                                } else {
                                    // we don't have a right parenthesis!
                                    // we need to report an error
                                    return Err(ParserError::new(
                                        "A right parenthesis is expected here.".to_string(),
                                        "Expected a right parenthesis to follow a function argument declaration.".to_string(),
                                        self.tokens.first().unwrap().range(),
                                        self.body.clone(),
                                        None
                                    ));
                                }
                            }
                        } else {
                            return Err(ParserError::new(
                                "A type statement is expected here.".to_string(),
                                "Expected a type statement to follow a function declaration argument.".to_string(),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None
                            ));
                        }
                    } else {
                        return Err(ParserError::new(
                            "A type statement is expected here.".to_string(),
                            "Expected a type statement to follow a function declaration argument."
                                .to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                } else {
                    return Err(ParserError::new(
                        "A name is expected here.".to_string(),
                        "Expected a function parameter name but none was found.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }

            let mut returns: Option<TypeKind> = None;

            // we're outside the function input list now, we need to check for a colon, again
            // if there is none, "void" is assumed
            if let Some(_) = self.tokens.peek_if(|t| t.kind().is_colon()) {
                // we need to parse a type statement
                self.skip_whitespace_err(
                    "Expected a return type statement after a function declaration.",
                )?;
                if let Some(type_smt) = self.parse_type_kind()? {
                    returns = Some(type_smt);
                } else {
                    return Err(ParserError::new(
                        "Expected a return type statement to follow a function declaration."
                            .to_string(),
                        "A return type is expected here.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }

            return Ok(Some((inputs, returns)));
        }
        return Ok(None);
    }

    /// Parses any class declaration.
    fn parse_class(&mut self) -> Result<Option<Class>, ParserError> {
        if let Some(_) = self
            .tokens
            .peek_if(|t| t.kind().is_keyword() && (t.kind().as_keyword() == KeyWord::Class))
        {
            self.skip_whitespace();
            if let Some(name) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                self.skip_whitespace();
                let extends = self.parse_class_extension()?;
                self.skip_whitespace();
                let implements: Option<Vec<String>> = self.parse_class_implementation()?;
                let body: Option<ClassBody> = self.parse_class_body()?;
                return Ok(Some(Class {
                    name: name.value().unwrap(),
                    extends,
                    implements,
                    body: body.unwrap_or(ClassBody::new()),
                    node_id: self.context.clone().get_next_local_id(),
                }));
            } else {
                return Err(ParserError::new(
                    format!(
                        "Unexpected token: {}",
                        self.tokens.first().unwrap().kind().to_string()
                    ),
                    "Expected a class name but none was found.".to_string(),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        } else {
            return Ok(None);
        }
    }

    fn parse_class_extension(&mut self) -> Result<Option<String>, ParserError> {
        if let Some(_) = self
            .tokens
            .peek_if(|t| t.kind().is_keyword() && (t.kind().as_keyword() == KeyWord::Extends))
        {
            self.skip_whitespace();
            if let Some(path) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                return Ok(Some(path.value().unwrap()));
            } else {
                return Err(ParserError::new(
                    format!(
                        "Unexpected token: {}",
                        self.tokens.first().unwrap().kind().to_string()
                    ),
                    "Expected a class name to extend but none was found.".to_string(),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        }
        return Ok(None);
    }

    fn parse_class_implementation(&mut self) -> Result<Option<Vec<String>>, ParserError> {
        if let Some(_) = self
            .tokens
            .peek_if(|t| t.kind().is_keyword() && (t.kind().as_keyword() == KeyWord::Implements))
        {
            self.skip_whitespace();
            if let Some(path) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                let mut paths: Vec<String> = vec![path.value().unwrap()];
                while !self.tokens.is_eof() {
                    self.skip_whitespace();
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                        self.skip_whitespace();
                        if let Some(path) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                            paths.push(path.value().unwrap());
                        } else {
                            return Err(ParserError::new(
                                format!(
                                    "Unexpected token: {}",
                                    self.tokens.first().unwrap().kind().to_string()
                                ),
                                "Expected a class name to extend but none was found.".to_string(),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None,
                            ));
                        }
                    } else {
                        break;
                    }
                }

                if !self.tokens.is_eof() {
                    return Ok(Some(paths));
                } else {
                    return Err(ParserError::new(
                        format!(
                            "Unexpected token: {}",
                            self.tokens.first().unwrap().kind().to_string()
                        ),
                        "Expected a class name or interface to implement but none was found."
                            .to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            } else {
                return Err(ParserError::new(
                    "Expected a class name to implement but none was found.".to_string(),
                    format!(
                        "Unexpected token: {}",
                        self.tokens.first().unwrap().kind().to_string()
                    ),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        }
        return Ok(None);
    }

    /// This function will attempt to parse a class property, however
    /// it will not parse it if it is not a property.
    fn parse_class_property(
        &mut self,
        visibility: Visibility,
    ) -> Result<Option<ClassProperty>, ParserError> {
        if let Some(name) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
            let mut type_node: Option<TypeKind> = None;
            // check if there's a type assigned to the property, if not, check for a statement end.
            if let Some(_) = self.tokens.peek_if(|t| t.kind().is_colon()) {
                // type statement.
                self.skip_whitespace();
                if let Some(kind) = self.parse_type_kind()? {
                    type_node = Some(kind);
                } else {
                    return Err(ParserError::new(
                        "A type statement is expected here.".to_string(),
                        "Expected a type statement to follow a property declaration.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }

            // check for an "equals" operator
            if let Some(_) = self
                .tokens
                .peek_if(|t| t.kind().is_operator() && (t.value().unwrap() == "=".to_string()))
            {
                // we have an equals operator!
                // we need to parse an expression
                self.skip_whitespace_err("An expression was expected but none was found.")?;
                if let Some(expr) = self.parse_expression()? {
                    // we have an expression!
                    // we need to parse a semicolon
                    self.skip_whitespace_err("A semicolon was expected but none was found.")?;
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                        return Ok(Some(ClassProperty::new(
                            name.value().unwrap(),
                            visibility,
                            type_node.clone(),
                            Some(expr),
                        )));
                    } else {
                        return Err(ParserError::new(
                            "Expected a semicolon to follow a variable declaration.".to_string(),
                            "A semicolon is expected here.".to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                } else {
                    return Err(ParserError::new(
                        "An expression is expected here.".to_string(),
                        "Expected an expression to follow a variable declaration.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            } else {
                // variables **can** be uninitialized
                // we need to check if the next token is an end of statement
                if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                    // we have an end of statement!
                    // we can return a variable declaration
                    return Ok(Some(ClassProperty::new(
                        name.value().unwrap(),
                        visibility,
                        type_node.clone(),
                        None,
                    )));
                } else {
                    // we don't have an end of statement!
                    // we need to report an error
                    return Err(ParserError::new(
                        "A semi-colon is expected here.".to_string(),
                        "Expected an end of statement to follow an uninitialized declaration."
                            .to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }
        }
        return Ok(None);
    }

    fn parse_class_allowed_statement(
        &mut self,
    ) -> Result<Option<ClassAllowedStatement>, ParserError> {
        // check for visibility
        let visibility = self.parse_visibility()?.unwrap_or(Visibility::Private);
        if let Some(_) = self
            .tokens
            .peek_if(|t| t.kind().is_keyword() && t.kind().as_keyword() == KeyWord::Static)
        {
            self.skip_whitespace();
            // the statement is static
            if let Some(property) = self.parse_class_property(visibility.clone())? {
                return Ok(Some(ClassAllowedStatement::new_static(
                    ClassAllowedStatement::Property(property),
                )));
            } else if let Some(mut func) = self.parse_function()? {
                func.visibility = visibility;
                return Ok(Some(ClassAllowedStatement::new_static(
                    ClassAllowedStatement::Method(func),
                )));
            } else {
                return Err(ParserError::new(
                    format!(
                        "Unexpected token: {}",
                        self.tokens.first().unwrap().kind().to_string()
                    ),
                    "Expected a property or function declaration but none was found.".to_string(),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        } else {
            // the statement is not static
            // Parse a property
            self.skip_whitespace_err("Expected a class statement but none was found.")?;
            if let Some(property) = self.parse_class_property(visibility.clone())? {
                return Ok(Some(ClassAllowedStatement::Property(property)));
            } else if let Some(mut func) = self.parse_function()? {
                func.visibility = visibility;
                return Ok(Some(ClassAllowedStatement::Method(func)));
            } else {
                return Err(ParserError::new(
                    "Expected a property or function declaration but none was found.".to_string(),
                    format!(
                        "Unexpected token: {}",
                        self.tokens.first().unwrap().kind().to_string()
                    ),
                    self.tokens.first().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        }
    }

    fn parse_class_body(&mut self) -> Result<Option<ClassBody>, ParserError> {
        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_left_brace()) {
            let mut body = ClassBody::new();
            // opening a body.
            // we need to parse the body until we reach the end
            while !self.tokens.is_eof()
                && !self
                    .tokens
                    .first_if(|t| t.kind().is_right_brace())
                    .is_some()
            {
                self.skip_whitespace_err(
                    "Expected a right brace to close the class body, found none.",
                )?;
                if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_brace()) {
                    break;
                } else if let Some(property) = self.parse_class_property(Visibility::Private)? {
                    body.properties.push(property);
                } else if let Some(method) = self.parse_function()? {
                    body.methods.push(method);
                } else if let Some(other) = self.parse_class_allowed_statement()? {
                    body.other.push(other);
                } else {
                    return Err(ParserError::new(
                        format!(
                            "Unexpected token: \"{}\" inside class body.",
                            self.tokens.first().unwrap().kind().to_string()
                        ),
                        "Classes must contain a property, method, import or macro.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }

            return Ok(Some(body));
        } else {
            return Ok(None);
        }
    }

    /// Parses any block statement
    /// A block statement is a statement that is surrounded by curly braces
    /// However, this does not include class bodies, as they have special properties.
    fn parse_block(&mut self) -> Result<Option<Vec<Expression>>, ParserError> {
        // we're expecting the next token to be a brace
        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_left_brace()) {
            // we have a brace!
            // we need to parse the statements inside the block
            let mut expressions: Vec<Expression> = Vec::new();
            while !self.tokens.is_eof() {
                self.skip_whitespace_err("Expected a statement to follow a block.")?;
                if let Some(expr) = self.parse_expression()? {
                    expressions.push(expr);
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_brace()) {
                    // we have a right brace!
                    // this is the end of the block.
                    break;
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                    expressions.push(Expression::EndOfLine);
                } else if let Some(_) = self
                    .tokens
                    .peek_if(|t| t.kind().is_keyword() && t.kind().as_keyword() == KeyWord::Return)
                {
                    // we have a return statement!
                    // we need to parse the return statement
                    self.skip_whitespace();
                    if let Some(expr) = self.parse_expression()? {
                        expressions.push(Expression::Statement(Box::new(Statement::Return(
                            Return::new(Some(expr)),
                        ))));
                    }
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_statement_end()) {
                        // end of statement! however, we dont care because this is a block and we don't
                        // have the context of the block.
                        continue;
                    } else {
                        return Err(ParserError::new(
                            "Expected an expression here.".to_string(),
                            "Expected an expression to follow a return statement.".to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                } else {
                    return Err(ParserError::new(
                        "A statement is expected here.".to_string(),
                        "Expected a statement to follow a block.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }
            return Ok(Some(expressions));
        } else {
            return Ok(None);
        }
    }

    /// Parses a visibility keyword
    /// If a `static` keyword follows the visibility, the statement SHOULD be static.
    /// > This is an alias for `parse_statement` as it will only parse visibility and static statements.
    /// EG: `public`
    /// EG: `private static`
    fn parse_visibility(&mut self) -> Result<Option<Visibility>, ParserError> {
        if let Some(modifier) = self
            .tokens
            .peek_if(|t| t.kind().is_keyword() && t.kind().as_keyword().is_visibility())
        {
            let visibility = Visibility::from_keyword(modifier.kind().as_keyword());

            self.skip_whitespace_err("A statement or static keyword was expected after a visibility modifier but none was found.")?;

            return Ok(Some(visibility));
        } else {
            return Ok(None);
        }
    }

    /// Similar to `parse_statement` but it will not consume the token stream,
    // fn get_visibility(&mut self) -> Option<Visibility> {
    //     if let Some(modifier) = self
    //         .tokens
    //         .first_if(|t| t.kind().is_keyword() && t.kind().as_keyword().is_visibility())
    //     {
    //         let visibility = Visibility::from_keyword(modifier.kind().as_keyword());

    //         return Ok(Some(visibility));
    //     } else {
    //         return Ok(None);
    //     }
    // }

    /// Parses a type kind.
    /// For example:
    /// - `int`
    /// - `string`
    /// - `bool`
    fn parse_type_kind(&mut self) -> Result<Option<TypeKind>, ParserError> {
        if let Some(initial) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
            let name = initial.value().unwrap();
            // The first token is an identifier! This is good, this is a type kind already, however!,
            // we need to check if the next token is a union, if it's not, we can return the type kind.
            self.skip_whitespace();
            if let Some(_) = self
                .tokens
                .peek_if(|t| t.kind().is_operator() && t.value().unwrap().as_str() == "|")
            {
                // this is a union type!
                let mut union_type = TypeUnion::empty();
                while !self.tokens.is_eof() {
                    // we need to recursively parse in a union type, this can be exhausting!
                    // because of this, we will only be parsing type references here.
                    self.skip_whitespace_err("Expected a type reference to follow a union type.")?;
                    if let Some(_) = self
                        .tokens
                        .peek_if(|t| t.kind().is_operator() && t.value().unwrap().as_str() == "|")
                    {
                        // we have another pipe, meaning another type to the type union, lets parse the next token.
                        self.skip_whitespace_err(
                            "Expected a type reference to follow a union type.",
                        )?;
                        if let Some(name) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                            // we have a type reference!
                            union_type
                                .types
                                .push(TypeKind::Reference(TypeReference::new(
                                    name.value().unwrap(),
                                    self.parse_type_generics()?,
                                )));
                        } else {
                            return Err(ParserError::new(
                                "A type reference is expected here.".to_string(),
                                "Expected a type reference to follow a union type.".to_string(),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None,
                            ));
                        }
                    } else if let Some(_) =
                        self.tokens.first_if(|t| t.value().unwrap().as_str() == "=")
                    {
                        // we have an equals sign, meaning this union is completed.
                        break;
                    } else {
                        return Err(ParserError::new(
                            "A type reference is expected here.".to_string(),
                            "Expected a type reference to follow a union type.".to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                }

                // check to see if all types are actually references in the union.
                // basically checking if the reference is a builtin.
                for type_kind in union_type.types.iter_mut() {
                    if let TypeKind::Reference(ref reference) = type_kind {
                        if let Some(built_in) = BuiltInType::from_string(reference.name.clone()) {
                            *type_kind = TypeKind::BuiltIn(built_in);
                        }
                    }
                }
                return Ok(Some(TypeKind::Union(Box::new(union_type))));
            } else {
                if let Some(ty) = BuiltInType::from_string(name.clone()) {
                    return Ok(Some(TypeKind::BuiltIn(ty)));
                } else {
                    return Ok(Some(TypeKind::Reference(TypeReference::new(
                        name.clone(),
                        self.parse_type_generics()?,
                    ))));
                }
            }
        }
        return Ok(None);
    }

    fn parse_type_generics(&mut self) -> Result<Option<Vec<TypeParam>>, ParserError> {
        if let Some(_) = self
            .tokens
            .peek_if(|t| t.kind().is_operator() && t.value().unwrap() == "<")
        {
            let mut generics: Vec<TypeParam> = Vec::new();
            while !self.tokens.is_eof() {
                self.skip_whitespace_err(
                    "Expected a type paramater to follow a typed parameter list.",
                )?;
                if let Some(kind) = self.parse_type_kind()? {
                    generics.push(TypeParam::new(kind));
                } else if let Some(_) = self
                    .tokens
                    .peek_if(|t| t.kind().is_operator() && t.value().unwrap() == ">")
                {
                    // check if the generics list is empty, if so throw an error
                    if generics.is_empty() {
                        return Err(ParserError::new(
                            "A type paramater is expected here.".to_string(),
                            "Expected a type paramater to follow a typed parameter list."
                                .to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    } else {
                        return Ok(Some(generics));
                    }
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                    continue;
                } else {
                    return Err(ParserError::new(
                        "A type paramater is expected here.".to_string(),
                        "Expected a type paramater to follow a typed parameter list.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }
        }

        return Ok(None);
    }

    /// Parses an expression.
    /// For example:
    /// - `5`
    /// - `x`
    /// - `x + 5`
    /// - `x + 5 * y`
    fn parse_expression(&mut self) -> Result<Option<Expression>, ParserError> {
        // We're storing this operand in a variable so we can return it later.
        // We will be using this to parse operations.
        let mut left: Option<Expression> = None;

        // parse a statement expression
        // this needs to be before object parsing because
        // object expressions will assume a block check has already taken place.
        if let Some(statement_expr) = self.parse_statement()? {
            left = Some(Expression::Statement(Box::new(statement_expr)));
        }

        // parse a call expression
        if let Some(call_expr) = self.parse_call_expression()? {
            left = Some(Expression::Call(call_expr));
        }

        // parse a member expression
        if let Some(member_expr) = self.parse_member_expression()? {
            left = Some(Expression::Member(member_expr));
        }

        // parse a new expression
        if let Some(new_expr) = self.parse_new_expression()? {
            left = Some(Expression::New(new_expr));
        }

        // parse an array
        if let Some(array_expr) = self.parse_array_expression()? {
            left = Some(Expression::Array(array_expr));
        }

        if let Some(object_expr) = self.parse_object_expression()? {
            left = Some(Expression::Object(object_expr));
        }

        if let Some(literal_expr) = self.parse_literal_expression()? {
            left = Some(Expression::Literal(literal_expr));
        }

        // check left
        if let Some(left) = left {
            self.skip_whitespace();
            // check whitespace
            if let Some(ops) = self.tokens.peek_if(|t| t.kind().is_operator()) {
                self.skip_whitespace();
                if let Some(op) = AnyOperation::from_string(ops.value().unwrap()) {
                    // we have an operation!
                    self.skip_whitespace();
                    if let Some(right) = self.parse_expression()? {
                        let instruction = Operation::new(left, op, right);
                        return Ok(Some(Expression::Operation(instruction)));
                    } else {
                        return Err(ParserError::new(
                            "An expression is expected here.".to_string(),
                            "Expected an expression to follow an operation.".to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                } else {
                    return Err(ParserError::new(
                        "Unknown operator: {}".to_string(),
                        ops.value().unwrap(),
                        ops.range(),
                        self.body.clone(),
                        None,
                    ));
                }
            } else {
                return Ok(Some(left));
            }
        } else {
            return Ok(None);
        }
    }

    fn parse_call_expression(&mut self) -> Result<Option<Call>, ParserError> {
        // parse a call expression
        if let Some(identifier) = self.tokens.first_if(|t| t.kind().is_identifier()) {
            // we have an identifier, we need to try to parse function arguments now.
            if let Some(args) = self.parse_function_call_inputs()? {
                // This is definitely a function call.
                return Ok(Some(Call::new(identifier.value().unwrap(), args)));
            } else {
                // This probably isn't a function call.
                return Ok(None);
            }
        }

        return Ok(None);
    }

    fn parse_member_expression(&mut self) -> Result<Option<MemberListNode>, ParserError> {
        // parse a member expression
        if let Some(identifier) = self.tokens.first_if(|t| t.kind().is_identifier()) {
            // we have an identifier, we need to try to parse member expressions now.
            // we need to verify that this is a member expression
            // we need to check if the next token is a period

            if let Some(accessor) = self.tokens.second_if(|t| t.kind().is_accessor()) {
                let access_kind = match accessor.value().unwrap().as_str() {
                    "." => MemberLookup::Dynamic,
                    "::" => MemberLookup::Static,
                    _ => unreachable!(),
                };

                self.tokens.peek_inc(2);
                // we have a period, we need to parse a member expression
                // we need to parse a member expression
                if let Some(member_expr) = self.parse_expression()? {
                    // we have a member expression, we need to create a member list node
                    println!("Parsed a member node!!");
                    return Ok(Some(MemberListNode::new(
                        member_expr,
                        identifier.clone(),
                        access_kind,
                    )));
                } else {
                    // we don't have a member expression, we need to report an error
                    return Err(ParserError::new(
                        "An expression was expected here.".to_string(),
                        "Expected an expression to follow a property member.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            } else {
                // we don't have a period, this is probably not a member expression
                return Ok(None);
            }
        }

        return Ok(None);
    }

    fn parse_new_expression(&mut self) -> Result<Option<NewCall>, ParserError> {
        if let Some(_) = self
            .tokens
            .first_if(|t| t.kind().is_keyword() && t.kind().as_keyword().is_new())
        {
            // we have a new keyword, we need to parse a name.
            if let Some((inc, name)) = self.tokens.find_after_nth(
                1,
                |t| t.kind().is_identifier(),
                |t| t.kind().is_whitespace(),
            ) {
                self.tokens.peek_inc(inc);
                // we have a name, we need to parse a function call inputs.
                if let Some(args) = self.parse_function_call_inputs()? {
                    // we have a function call inputs, we need to create a new call.
                    return Ok(Some(NewCall::new(name.value().unwrap(), args)));
                } else {
                    // we don't have a function call inputs, we need to report an error.
                    return Err(ParserError::new(
                        "Function inputs expected here.".to_string(),
                        "Expected a function call inputs to follow a new expression.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            } else {
                // we don't have a name, we need to report an error.
                return Err(ParserError::new(
                    "A name was expected here.".to_string(),
                    "Expected a name to follow a new expression.".to_string(),
                    self.tokens.second().unwrap().range(),
                    self.body.clone(),
                    None,
                ));
            }
        }
        return Ok(None);
    }

    fn parse_array_expression(&mut self) -> Result<Option<Array>, ParserError> {
        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_left_bracket()) {
            // inside array
            let mut elements: Vec<Expression> = Vec::new();
            while !self.tokens.is_eof() {
                self.skip_whitespace_err("Array's must be closed.")?;
                if let Some(element) = self.parse_expression()? {
                    // we have an expression, we need to parse a comma
                    self.skip_whitespace_err("Array's must be closed.")?;
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                        elements.push(element);
                    } else {
                        // ok, check if the next token is a right bracket, if so, we're done.
                        // otherwise error
                        self.skip_whitespace_err("Array's must be closed.")?;
                        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_bracket()) {
                            // we have a right bracket, we can return the inputs
                            elements.push(element);
                            return Ok(Some(Array::new(elements, None)));
                        } else {
                            return Err(ParserError::new(
                                "A comma is expected here.".to_string(),
                                "A comma is required to seperate array elements.".to_string(),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None,
                            ));
                        }
                    }
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_bracket()) {
                    // end of array
                    return Ok(Some(Array::new(elements, None)));
                } else {
                    // we don't have an expression, we need to report an error.
                    return Err(ParserError::new(
                        format!(
                            "Unexpected Token: {}",
                            self.tokens.first().unwrap().kind().to_string()
                        ),
                        "Expected an expression to follow an array element.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }
        }
        return Ok(None);
    }

    fn parse_object_expression(&mut self) -> Result<Option<Object>, ParserError> {
        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_left_brace()) {
            // this is definitely an object body.
            let mut object: Object = Object::empty();

            while !self.tokens.is_eof() {
                // purge whitespace.
                self.skip_whitespace_err("Object body must be closed.")?;
                if let Some(property) = self.tokens.peek_if(|t| t.kind().is_identifier()) {
                    // the property name was found, now we need to parse a colon.
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_colon()) {
                        // we have a colon, we need to parse an expression.
                        self.skip_whitespace_err("Object body must be closed.")?;
                        if let Some(expression) = self.parse_expression()? {
                            // we have an expression, we need to add the property to the object.
                            let prop = ObjectProperty::new(property.value().unwrap(), expression);

                            // check if we have a comma, if so, we need to parse another property.
                            // otherwise we need to check if we have a right brace, if so, we're done.
                            if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                                // we have a comma, we need to parse another property.
                                object.properties.push(prop);
                            } else {
                                // check for a right brace, if so, we're done.
                                self.skip_whitespace_err("Object body must be closed.")?;
                                if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_brace())
                                {
                                    // we have a right brace, we're done.
                                    object.properties.push(prop);
                                    return Ok(Some(object));
                                } else {
                                    // we don't have a right brace, we need to report an error.
                                    return Err(ParserError::new(
                                        "A right brace was expected here.".to_string(),
                                        "Expected a right brace to close an object body."
                                            .to_string(),
                                        self.tokens.first().unwrap().range(),
                                        self.body.clone(),
                                        None,
                                    ));
                                }
                            }
                        } else {
                            // we don't have an expression, we need to report an error.
                            return Err(ParserError::new(
                                "An expression was expected here.".to_string(),
                                "Expected an expression to follow a property.".to_string(),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None,
                            ));
                        }
                    } else {
                        // we don't have a colon, we need to report an error.
                        return Err(ParserError::new(
                            format!(
                                "Unexpected Token: {}",
                                self.tokens.first().unwrap().kind().to_string()
                            ),
                            "Expected a colon to follow a property name.".to_string(),
                            self.tokens.first().unwrap().range(),
                            self.body.clone(),
                            None,
                        ));
                    }
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_brace()) {
                    // end of object
                    return Ok(Some(object));
                } else {
                    // we don't have an object property, we need to report an error.
                    return Err(ParserError::new(
                        "An object property was expected here.".to_string(),
                        "Expected an object property to follow an object element.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }
        }
        return Ok(None);
    }

    fn parse_literal_expression(&mut self) -> Result<Option<Literal>, ParserError> {
        // we have a literal, we need to parse a value.
        // a literal is either a string, number, boolean or null
        // either way we need to check if the next token is a identifier.
        if let Some(v) = self.tokens.peek_if(|t| {
            t.kind().is_identifier()
                || t.kind().is_number()
                || t.kind().is_string()
                || t.kind().is_boolean()
        }) {
            return Ok(Some(Literal::new(v.value().unwrap(), None)));
        } else {
            return Ok(None);
        }
    }

    /// parses function inputs (aka arguments)
    fn parse_function_call_inputs(&mut self) -> Result<Option<Vec<Expression>>, ParserError> {
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
                self.skip_whitespace_err("Function arguments must be closed.")?;

                if let Some(expr) = self.parse_expression()? {
                    // we have an expression, we need to parse a comma
                    if let Some(_) = self.tokens.peek_if(|t| t.kind().is_comma()) {
                        inputs.push(expr);
                    } else {
                        // ok, check if the next token is a parenthises, if so, we're done.
                        // otherwise error
                        if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_parenthesis()) {
                            // we have a right parenthesis, we can return the inputs
                            inputs.push(expr);
                            return Ok(Some(inputs));
                        } else {
                            return Err(ParserError::new(
                                "A comma is expected here.".to_string(),
                                "Expected a comma to follow a function input.".to_string(),
                                self.tokens.first().unwrap().range(),
                                self.body.clone(),
                                None,
                            ));
                        }
                    }
                } else if let Some(_) = self.tokens.peek_if(|t| t.kind().is_right_parenthesis()) {
                    // we have a right parenthesis, we can return the inputs
                    return Ok(Some(inputs));
                } else {
                    // we don't have an expression, we need to report an error
                    return Err(ParserError::new(
                        "An expression is expected here.".to_string(),
                        "Expected an expression to follow a function input.".to_string(),
                        self.tokens.first().unwrap().range(),
                        self.body.clone(),
                        None,
                    ));
                }
            }

            return Err(ParserError::new(
                "An expression is expected here.".to_string(),
                "Expected an expression to follow a function input.".to_string(),
                self.tokens.first().unwrap().range(),
                self.body.clone(),
                None,
            ));
        }

        return Ok(None);
    }

    fn skip_whitespace_err(&mut self, err: &'static str) -> Result<(), ParserError> {
        let start = self.tokens.first().expect(err).range().start;
        match self
            .tokens
            .peek_until(|t| !t.kind().is_whitespace() && !t.kind().is_comment())
        {
            None => {
                return Err(ParserError::new(
                    err.to_string(),
                    format!("Whitespace terminated the code while parsing an expression or statement. Make sure you're closing your code blocks and shit :)"),
                    start..self.context.clone().source.get_contents().unwrap().len(),
                    self.body.clone(),
                    None,
                ));
            }
            _ => Ok(()),
        }
    }

    fn skip_whitespace(&mut self) {
        self.tokens.peek_until(|t| {
            if t.kind().is_whitespace() || t.kind().is_comment() {
                return false;
            } else {
                return true;
            }
        });
    }
}
