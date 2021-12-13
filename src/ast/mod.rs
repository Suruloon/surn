use crate::{lexer::keyword::KeyWord, types::TypeRef};

use self::ops::AnyOperation;
pub mod ops;

// Expressions {{

/// An expression is any thing that can be evaluated to a value.
/// For example:
///  - `x + 1`
///  - `some_function()`
#[derive(Debug, Clone)]
pub enum Expression {
    /// A regular function call.
    ///
    /// For example:
    /// - `some_function()`
    Call(Call),
    /// A method call.
    ///
    /// For example:
    /// - `x.method()`
    MethodCall(MethodCall),
    /// An array literal.
    ///
    /// For example:
    /// - `[1, 2, 3]`
    /// - `[1; 10]`
    Array(Array),
    /// An Object Literal.
    /// For example:
    /// - `{ key: "value" }`
    Object(Object),
    /// An Operation.
    ///
    /// For example:
    /// - `1 + 2`
    /// - `1 - 2`
    Operation(Operation),
    /// A statement
    Statement(Box<Statement>),
    /// A member expression
    ///
    /// For example:
    /// - `x.y`
    /// - `x[y]`
    /// - `x.y.z`
    /// - `x[y].z`
    /// - `x.y[z]`
    Member(MemberListNode),
    /// A literal value.
    ///
    /// For example:
    /// - `1`
    /// - `"hello"`
    /// - `true`
    /// - `false`
    Literal(Literal),
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: String,
    /// The type of the literal assumed by the compiler
    pub type_node: Option<TypeRef>,
}

/// A member list is a list of members.
/// For example:
/// - `x.y`
///
/// The `name` is the value of the last member.
/// The `origin` is the value that the prop is coming from.
#[derive(Debug, Clone)]
pub struct MemberListNode {
    pub name: String,
    pub origin: Box<Expression>,
}

/// An array literal. This represents an array of values.
/// The values in the array are validated after parsing.
/// For example:
/// - `[1, 2, 3]`
/// - `[1; 10]`
#[derive(Debug, Clone)]
pub struct Array {
    pub values: Vec<Expression>,
    pub type_node: Option<TypeRef>,
}

#[derive(Debug, Clone)]
pub struct Object {
    /// The properties of the object.
    pub properties: Vec<ObjectProperty>,
    /// The type of the object.
    /// This is used to validate the object.
    /// However it can be None if the object is annonymous.
    pub type_node: Option<TypeRef>,
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    /// The name of the property.
    pub name: String,
    /// The value of the property.
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub op: AnyOperation,
}
// }}

// Statements {{
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
    Var(Variable),
    /// A const statement.
    Const(Variable),
    /// A static statement.
    Static(Box<Statement>),
    /// A function declaration.
    Function(Function),
    /// A class declaration.
    Class(Class),
    /// A block statment
    Block(Vec<Expression>),
    /// A import statement.
    /// For example:
    /// - `use foo;`
    /// - `use foo::bar;`
    /// - `use foo::bar::baz;`
    /// - `use foo::bar as baz;`
    /// - `use foo::bar::{ baz, bat };`
    Import(Path),
    /// A type statement / alias.
    /// For example:
    /// - `type Foo = int;`
    /// - `type Foo = Bar;`
    Type(TypeRef),
    /// A macro invocation.
    /// For example:
    /// - `php!( "hello" )`
    /// - `php! { public function foo() { return "hello"; } }`
    MacroInvocation(CompilerMacro),
}

impl Statement {
    pub fn get_block(&self) -> Option<Vec<Expression>> {
        match self {
            Statement::Block(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get_type(&self) -> Option<TypeRef> {
        match self {
            Statement::Type(t) => Some(t.clone()),
            _ => None,
        }
    }

    pub fn get_function(&self) -> Option<Function> {
        match self {
            Statement::Function(f) => Some(f.clone()),
            _ => None,
        }
    }

    pub fn get_class(&self) -> Option<Class> {
        match self {
            Statement::Class(c) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn get_import(&self) -> Option<Path> {
        match self {
            Statement::Import(p) => Some(p.clone()),
            _ => None,
        }
    }

    pub fn get_macro_invocation(&self) -> Option<CompilerMacro> {
        match self {
            Statement::MacroInvocation(m) => Some(m.clone()),
            _ => None,
        }
    }

    pub fn get_mutable(&self) -> Option<Variable> {
        match self {
            Statement::Var(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get_immutable(&self) -> Option<Variable> {
        match self {
            Statement::Const(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get_static(&self) -> Option<Box<Statement>> {
        match self {
            Statement::Static(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn is_block(&self) -> bool {
        match self {
            Statement::Block(_) => true,
            _ => false,
        }
    }

    pub fn is_class(&self) -> bool {
        match self {
            Statement::Class(_) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Statement::Function(_) => true,
            _ => false,
        }
    }

    pub fn is_import(&self) -> bool {
        match self {
            Statement::Import(_) => true,
            _ => false,
        }
    }

    pub fn is_macro_invocation(&self) -> bool {
        match self {
            Statement::MacroInvocation(_) => true,
            _ => false,
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Statement::Var(_) => true,
            _ => false,
        }
    }

    pub fn is_immutable(&self) -> bool {
        match self {
            Statement::Const(_) => true,
            _ => false,
        }
    }

    pub fn is_type(&self) -> bool {
        match self {
            Statement::Type(_) => true,
            _ => false,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Statement::Static(_) => true,
            _ => false,
        }
    }
}
// }}

// Visibility {{
#[derive(Debug, Clone)]
pub enum Visibility {
    /// Public visibility. Every module can see this.
    Public,
    /// Private visibility. Only the current **scope** can see this.
    Private,
    /// Protected, only the current **scope** and its children can see this.
    Protected,
    /// Internal visibility. Only the current **module** can see this.
    /// This is the default visibility.
    /// This is not userdefined.
    Module,
}

impl Visibility {
    pub fn from_keyword(keyword: KeyWord) -> Self {
        match keyword {
            KeyWord::Public => Visibility::Public,
            KeyWord::Private => Visibility::Private,
            KeyWord::Protected => Visibility::Protected,
            _ => Visibility::Private,
        }
    }
}
//}}

// Classes {{
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub properties: Vec<ClassProperty>,
    /// These are static properties.
    pub external: Vec<ClassProperty>,
    pub body: Vec<Statement>,
    pub node_id: u64,
}

impl Class {
    pub fn new(visibility: Visibility) -> Self {
        Class {
            name: String::new(),
            extends: None,
            body: Vec::new(),
            node_id: 0,
            properties: Vec::new(),
            external: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassProperty {
    pub name: String,
    pub visibility: Visibility,
    pub type_ref: TypeRef,
    pub value: Option<Expression>,
}

pub enum ClassBody {}
// }}

// Functions {{
/// A function call or method call.
#[derive(Debug, Clone)]
pub struct Function {
    /// The name of the function.
    pub name: String,
    /// The arguments to the function.
    pub inputs: Vec<FunctionInput>,
    /// The body of the function,
    pub body: Vec<Statement>,
    /// The return types of the function,
    pub outputs: Vec<TypeRef>,
    /// The visibilty of the function.
    pub visibility: Visibility,
    /// The id for the given function.
    pub node_id: u64,
}

#[derive(Debug, Clone)]
pub struct FunctionInput {
    pub name: String,
    pub types: Vec<TypeRef>,
}

/// A function call or method call.
/// This is calling a specific function.
/// For example:
/// - `foo()`
#[derive(Debug, Clone)]
pub struct Call {
    /// The name of the function being called.
    /// This is the name of the function, not the name of the variable.
    pub name: String,
    /// The arugments being passed to the function.
    pub arguments: Vec<Expression>,
}

/// A method call.
/// For example:
/// - `foo.bar()`
#[derive(Debug, Clone)]
pub struct MethodCall {
    /// The name of the function being called.
    /// This is the name of the function, not the name of the variable.
    pub name: String,
    /// The arugments being passed to the function.
    pub arguments: Vec<Expression>,
    /// The callee of the method call.
    pub callee: Box<Expression>,
}
// }}

// Variables & Types {{
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub node_id: u64,
    pub type_ref: TypeRef,
    pub visibility: Visibility,
    pub assignment: Option<Expression>,
}

impl Variable {
    pub fn new(
        name: String,
        type_ref: TypeRef,
        visibility: Visibility,
        assignment: Option<Expression>,
    ) -> Self {
        Self {
            name,
            node_id: 0,
            type_ref,
            visibility,
            assignment,
        }
    }

    pub fn is_uninit(&self) -> bool {
        self.assignment.is_none()
    }

    pub fn to_class_property(&self) -> ClassProperty {
        ClassProperty {
            name: self.name.clone(),
            visibility: self.visibility.clone(),
            type_ref: self.type_ref.clone(),
            value: self.assignment.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    /// The module to import.
    /// For example:
    /// - `foo`
    /// - `std` in `std::io` etc.
    pub name: String,
    /// The parts of the import
    /// For example:
    /// - `foo` in `bar::foo`
    /// - `bar, baz` in `foo::{bar, baz}`
    pub parts: Vec<Path>,
}
// }}

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

// AST {{
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
//}}
