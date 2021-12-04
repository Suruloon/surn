// Expressions {{

/// An expression is any thing that can be evaluated to a value.
/// For example:
///  - `x + 1`
///  - `some_function()`
#[derive(Debug, Clone)]
pub enum Expression {
    /// A function call.
    Call,
    /// An array literal.
    /// A statement
    Statement(Box<Statement>),
    /// A literal value.
    /// For example:
    /// - `1`
    /// - `"hello"`
    /// - `true`
    /// - `false`
    Literal(Literal),
    /// A macro invocation.
    /// For example:
    /// - `php!( "hello" )`
    /// - `php! { public function foo() { return "hello"; } }`
    MacroInvocation(CompilerMacro),
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: String,
    /// The type of the literal assumed by the compiler
    pub type_id: Option<u64>,
}
// }}

/// A statement is anything that can be executed.
/// For example:
/// - `x = 1`
/// - `return 1`
/// - `if (x > 1) { return 1; }`
/// - `class Foo {}`
/// - `type Foo = int;`
/// - `interface Foo {}`
#[derive(Debug, Clone)]
pub enum Statement {
    /// A var statement.
    Mutable(Variable),
    /// A const statement.
    Immutable(Variable),
    /// A function declaration.
    Function(Function),
    /// A class declaration.
    Class(Class),
    /// A block statment
    Block(Vec<Expression>),
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ClassProperty {
    pub name: String,
    pub visibility: Visibility,
    pub type_id: u64,
    pub value: Option<Expression>,
}

pub enum ClassBody {
    
}

/// A function call or method call.
#[derive(Debug, Clone)]
pub struct Function {
    /// The name of the function.
    pub name: String,
    /// The arguments to the function.
    pub inputs: Vec<FunctionInput>,
    /// The id for the given function.
    pub node_id: u64,
}

#[derive(Debug, Clone)]
pub struct FunctionInput {
    pub name: String,
    pub type_id: u64,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub node_id: u64,
    pub type_id: u64,
    pub assignment: Option<Expression>,
}

impl Variable {
    pub fn is_uninit(&self) -> bool {
        self.assignment.is_none()
    }
}

// Macros {{
#[derive(Debug, Clone)]
pub struct CompilerMacro {
    /// The name of the macro to invoke,
    /// e.g. `php!`
    /// or `php! { ... }`
    /// Note that these can not be user-defined.
    pub name: String,
    /// The body of the macro, this will be traversed
    /// during macro invokations.
    pub body: String,
}
// }}

#[derive(Debug, Clone)]
pub struct AstBody {
    // todo: Compiler flags
    flags: u64,
    program: Vec<Expression>,
}

impl AstBody {
    pub fn new() -> Self {
        AstBody {
            flags: 0,
            program: Vec::new(),
        }
    }

    pub fn push_statement(&mut self, statement: Statement) {
        self.program
            .push(Expression::Statement(Box::new(statement)));
    }

    pub fn push_expression(&mut self, expression: Expression) {
        self.program.push(expression);
    }

    pub fn get_program(&self) -> &Vec<Expression> {
        &self.program
    }
}
